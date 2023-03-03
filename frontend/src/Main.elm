module Main exposing (..)

import Browser
import Element exposing (..)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input exposing (OptionState(..))
import Html exposing (Html)
import Html.Attributes as RawAttrs
import Http
import Json.Decode as D
import MatrixTheme
import QRCode
import Time


type alias Program_ =
    { id : Int
    , name : String
    }


type alias Status =
    { level : Int
    , name : String
    }


type alias UserStatus =
    { program : Program_
    , status : Status
    }


type alias Link =
    { program : String
    , status : String
    }


type alias Challenge =
    String


type alias AccessToken =
    String


type alias TokenType =
    String


type alias Auth =
    { accessToken : AccessToken
    , tokenType : TokenType
    }


type alias LnurlAuth =
    { lnurl : Lnurl
    , k1 : Challenge
    }


type AuthState
    = AuthenticateLoading
    | Authenticating LnurlAuth
    | Authenticated Auth
    | AuthenticateFailed


type alias Model =
    { authState : AuthState
    , searchText : String
    , programs : Maybe (List Program_)
    , statuses : Maybe (List Status)
    , selectedProgram : Maybe Program_
    , selectedStatus : Maybe Status
    , links : Maybe (List Link)
    }


type alias Lnurl =
    String


type Msg
    = GotSearchText String
    | UpdateAuthState LnurlAuth
    | LoadLnurlAuth (Result Http.Error ( Lnurl, Challenge ))
    | LoadAuth (Result Http.Error Auth)
    | LoadPrograms (Result Http.Error (List Program_))
    | LoadStatuses (Result Http.Error (List Status))
    | LoadLinks (Result Http.Error (List Link))
    | LoadUserStatuses (Result Http.Error (List UserStatus))
    | ChooseProgram Program_
    | ChooseStatus Status


lnurlDecoder : D.Decoder ( Lnurl, Challenge )
lnurlDecoder =
    D.map2 (\lnurl challenge -> ( lnurl, challenge ))
        (D.field "lnurl" D.string)
        (D.field "k1" D.string)


authorizationDecoder : D.Decoder Auth
authorizationDecoder =
    D.map2 Auth
        (D.field "access_token" D.string)
        (D.field "token_type" D.string)


programDecoder : D.Decoder Program_
programDecoder =
    D.map2 Program_ (D.field "id" D.int) (D.field "name" D.string)


programListDecoder : D.Decoder (List Program_)
programListDecoder =
    D.list programDecoder


statusDecoder : D.Decoder Status
statusDecoder =
    D.map2 Status (D.field "level" D.int) (D.field "name" D.string)


statusListDecoder : D.Decoder (List Status)
statusListDecoder =
    D.list statusDecoder


linkDecoder : D.Decoder Link
linkDecoder =
    D.map2 Link (D.field "program" D.string) (D.field "status" D.string)


linkListDecoder : D.Decoder (List Link)
linkListDecoder =
    D.list linkDecoder


userStatusDecoder : D.Decoder UserStatus
userStatusDecoder =
    D.map2 UserStatus
        (D.field "program" programDecoder)
        (D.field "status" statusDecoder)


userStatusListDecoder : D.Decoder (List UserStatus)
userStatusListDecoder =
    D.list userStatusDecoder


fetchLnurlAuth : Cmd Msg
fetchLnurlAuth =
    Http.get
        { url = "api/login"
        , expect = Http.expectJson LoadLnurlAuth lnurlDecoder
        }


fetchLnurlAuthState : LnurlAuth -> Cmd Msg
fetchLnurlAuthState { k1 } =
    Http.get
        { url = "api/login/" ++ k1
        , expect = Http.expectJson LoadAuth authorizationDecoder
        }


fetchPrograms : String -> Cmd Msg
fetchPrograms text =
    Http.get
        { url = "api/programs/search?text=" ++ text
        , expect = Http.expectJson LoadPrograms programListDecoder
        }


fetchStatuses : Program_ -> Cmd Msg
fetchStatuses program =
    let
        id =
            String.fromInt program.id

        url =
            "api/programs/" ++ id ++ "/statuses"
    in
    Http.get
        { url = url
        , expect = Http.expectJson LoadStatuses statusListDecoder
        }


fetchLinks : Program_ -> Status -> Cmd Msg
fetchLinks program status =
    let
        id =
            String.fromInt program.id

        level =
            String.fromInt status.level

        url =
            "api/programs/" ++ id ++ "/statuses/" ++ level ++ "/links"
    in
    Http.get
        { url = url
        , expect = Http.expectJson LoadLinks linkListDecoder
        }


fetchUserStatuses : Auth -> Cmd Msg
fetchUserStatuses { tokenType, accessToken } =
    Http.request
        { method = "GET"
        , headers =
            [ Http.header "Authorization" (tokenType ++ " " ++ accessToken)
            ]
        , url = "api/user/statuses"
        , expect = Http.expectJson LoadUserStatuses userStatusListDecoder
        , body = Http.emptyBody
        , timeout = Nothing
        , tracker = Nothing
        }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotSearchText text ->
            ( { model
                | searchText = text
                , selectedProgram = Nothing
                , statuses = Nothing
                , selectedStatus = Nothing
              }
            , fetchPrograms text
            )

        UpdateAuthState lnurlAuth ->
            ( model
            , fetchLnurlAuthState lnurlAuth
            )

        LoadLnurlAuth (Ok ( lnurl, k1 )) ->
            ( { model
                | authState = Authenticating { lnurl = lnurl, k1 = k1 }
              }
            , Cmd.none
            )

        LoadLnurlAuth (Err _) ->
            ( model, Cmd.none )

        LoadAuth (Ok auth) ->
            ( { model
                | authState =
                    Authenticated auth
              }
            , fetchUserStatuses auth
            )

        LoadAuth (Err _) ->
            ( model, Cmd.none )

        LoadPrograms (Ok programs) ->
            ( { model
                | programs = Just programs
              }
            , Cmd.none
            )

        LoadPrograms (Err _) ->
            ( model, Cmd.none )

        LoadStatuses (Ok statuses) ->
            ( { model
                | statuses = Just statuses
              }
            , Cmd.none
            )

        LoadStatuses (Err _) ->
            ( model, Cmd.none )

        LoadLinks (Ok links) ->
            ( { model
                | links = Just links
              }
            , Cmd.none
            )

        LoadLinks (Err _) ->
            ( model, Cmd.none )

        LoadUserStatuses (Ok userStatuses) ->
            let
                userStatus =
                    userStatuses |> List.head

                ( newModel, cmd ) =
                    userStatus
                        |> Maybe.map
                            (\{ program, status } ->
                                ( { model
                                    | selectedProgram = Just program
                                    , selectedStatus = Just status
                                    , programs = Just [ program ]
                                  }
                                , fetchStatuses program
                                )
                            )
                        |> Maybe.withDefault ( model, Cmd.none )
            in
            ( newModel
            , cmd
            )

        LoadUserStatuses (Err _) ->
            ( model, Cmd.none )

        ChooseProgram program ->
            ( { model
                | selectedProgram = Just program
                , searchText = program.name
              }
            , fetchStatuses program
            )

        ChooseStatus status ->
            let
                fetchCmd =
                    Maybe.map2 fetchLinks model.selectedProgram (Just status)
                        |> Maybe.withDefault Cmd.none
            in
            ( { model
                | selectedStatus = Just status
              }
            , fetchCmd
            )


initialModel : Model
initialModel =
    { authState = AuthenticateLoading
    , searchText = ""
    , programs = Nothing
    , statuses = Nothing
    , selectedProgram = Nothing
    , selectedStatus = Nothing
    , links = Nothing
    }


init : () -> ( Model, Cmd Msg )
init () =
    ( initialModel, fetchLnurlAuth )


subscriptions : Model -> Sub Msg
subscriptions model =
    case model.authState of
        Authenticating lnurlAuth ->
            Time.every 1000 (always <| UpdateAuthState lnurlAuth)

        _ ->
            Sub.none


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }


viewProgram : Bool -> Program_ -> Element Msg
viewProgram isSelected program =
    let
        asString =
            if isSelected then
                "[X] " ++ program.name

            else
                "[ ] " ++ program.name
    in
    Input.button
        [ Font.color MatrixTheme.foregroundColor2
        , focused []
        ]
        { onPress = Just <| ChooseProgram program
        , label = text asString
        }


viewProgramList : Maybe Program_ -> List Program_ -> Element Msg
viewProgramList selectedProgram programs =
    let
        children =
            programs
                |> List.take 3
                |> List.map (\p -> viewProgram (Just p == selectedProgram) p)
    in
    Element.column
        [ paddingXY 0 8
        ]
        children


viewStatus : Status -> Input.Option Status msg
viewStatus status =
    let
        viewOption state =
            case state of
                Idle ->
                    text <| "[ ] " ++ status.name

                Focused ->
                    text <| "[ ] " ++ status.name

                Selected ->
                    text <| "[X] " ++ status.name
    in
    Input.optionWith status viewOption


viewStatusList : Maybe Status -> List Status -> Element Msg
viewStatusList selected statuses =
    Input.radio
        [ Font.color MatrixTheme.foregroundColor2
        , paddingXY 0 8
        ]
        { onChange = ChooseStatus
        , selected = selected
        , label = Input.labelAbove [ Font.bold ] <| text "STATUS: "
        , options = statuses |> List.map viewStatus
        }


viewForm : Model -> Element Msg
viewForm model =
    Element.column
        [ width fill
        , Border.color MatrixTheme.foregroundColor
        , Border.width 1
        , padding 16
        ]
        [ Input.search
            [ Input.focusedOnLoad
            , width fill
            , Background.color MatrixTheme.backgroundColor
            , paddingXY 0 0
            , Border.color MatrixTheme.foregroundColor
            , Border.solid
            , Border.rounded 0
            , focused
                [ Element.alpha 1.0
                ]
            , htmlAttribute <| RawAttrs.style "caret-color" "transparent"
            , htmlAttribute <| RawAttrs.style "font-family" "inherit"
            , htmlAttribute <| RawAttrs.style "font-size" "100%"
            , Element.alpha 0.2
            , inFront <|
                Element.el
                    [ htmlAttribute <| RawAttrs.class "caret"
                    , moveDown 6.0
                    , moveRight (10.0 * toFloat (String.length model.searchText |> max 1))
                    , width (px 10)
                    , height (px 20)
                    , Background.color MatrixTheme.foregroundColor
                    ]
                    Element.none
            ]
            { onChange = GotSearchText
            , text = model.searchText
            , placeholder = Nothing
            , label = Input.labelLeft [ Font.bold ] <| text "PROGRAM: "
            }
        , model.programs
            |> Maybe.map (viewProgramList model.selectedProgram)
            |> Maybe.withDefault Element.none
        , model.statuses
            |> Maybe.map (viewStatusList model.selectedStatus)
            |> Maybe.withDefault Element.none
        ]


viewLink : Link -> Element Msg
viewLink link =
    text ("- " ++ link.program ++ "(" ++ link.status ++ ")")


viewEmptyLinkList : Element Msg
viewEmptyLinkList =
    text "Oops. No status match."


viewLinkList : List Link -> List (Element Msg)
viewLinkList links =
    if links |> List.isEmpty then
        [ viewEmptyLinkList ]

    else
        links
            |> List.map viewLink


viewQrcodeLoading : Element msg
viewQrcodeLoading =
    text "Loading..."


viewQrcode : LnurlAuth -> Element msg
viewQrcode { lnurl } =
    let
        toLink child =
            link
                [ Background.color (rgb255 255 255 255)
                , width fill
                ]
                { url = "lightning:" ++ lnurl
                , label = child
                }
    in
    lnurl
        |> QRCode.fromString
        |> Result.map (QRCode.toSvg [] >> html >> toLink)
        |> Result.withDefault (text "Error while encoding to QRCode.")


viewWelcome : AuthState -> Element Msg
viewWelcome authState =
    let
        welcome =
            """
 1. Enter the program name.
 2. Select a status.
 3. Find the best status match.

 or

 Login with Lightning
    """
    in
    Element.el
        [ Font.family [ Font.monospace ]
        , htmlAttribute <| RawAttrs.style "font-size" "1vw"
        ]
    <|
        column []
            [ text welcome
            , case authState of
                AuthenticateLoading ->
                    viewQrcodeLoading

                Authenticating lnurlAuth ->
                    viewQrcode lnurlAuth

                Authenticated _ ->
                    text "success"

                AuthenticateFailed ->
                    text "failure"
            ]


viewResult : Model -> Element Msg
viewResult model =
    let
        children =
            model.links
                |> Maybe.map viewLinkList
                |> Maybe.withDefault [ viewWelcome model.authState ]
    in
    Element.column
        [ Border.color MatrixTheme.foregroundColor
        , Border.width 1
        , padding 16
        , width fill
        , height fill
        ]
        children


view : Model -> Html Msg
view model =
    Element.layout
        [ htmlAttribute <| RawAttrs.style "font-feature-settings" "\"palt\"" ]
    <|
        Element.column
            [ Element.padding 16
            , Background.color MatrixTheme.backgroundColor
            , Font.color MatrixTheme.foregroundColor
            , Font.family [ MatrixTheme.font ]
            , height fill
            , width fill
            ]
            [ viewForm model
            , viewResult model
            ]
