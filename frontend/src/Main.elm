module Main exposing (..)

import Browser
import Element exposing (..)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input exposing (OptionState(..))
import Html exposing (Html)
import Html.Attributes as RawAttrs exposing (class)
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


type alias Link =
    { program : String
    , status : String
    }


type alias Challenge =
    String


type alias Model =
    { lnurl : Maybe Lnurl
    , k1 : Maybe Challenge
    , isAuth : Bool
    , authResult : Maybe Bool
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
    | AuthStatus
    | LoadLnurlAuth (Result Http.Error ( Lnurl, Challenge ))
    | LoadLnurlAuthResult (Result Http.Error Bool)
    | LoadPrograms (Result Http.Error (List Program_))
    | LoadStatuses (Result Http.Error (List Status))
    | LoadLinks (Result Http.Error (List Link))
    | ChooseProgram Program_
    | ChooseStatus Status


lnurlDecoder : D.Decoder ( Lnurl, Challenge )
lnurlDecoder =
    D.map2 (\lnurl challenge -> ( lnurl, challenge ))
        (D.field "lnurl" D.string)
        (D.field "k1" D.string)


lnurlResultDecoder : D.Decoder Bool
lnurlResultDecoder =
    D.field "result" D.bool


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


fetchLnurlAuth : Cmd Msg
fetchLnurlAuth =
    Http.get
        { url = "api/login"
        , expect = Http.expectJson LoadLnurlAuth lnurlDecoder
        }


fetchLnurlAuthResult : Challenge -> Cmd Msg
fetchLnurlAuthResult k1 =
    Http.get
        { url = "api/login/" ++ k1
        , expect = Http.expectJson LoadLnurlAuthResult lnurlResultDecoder
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

        AuthStatus ->
            ( model
            , model.k1
                |> Maybe.map fetchLnurlAuthResult
                |> Maybe.withDefault Cmd.none
            )

        LoadLnurlAuth (Ok ( lnurl, k1 )) ->
            ( { model
                | lnurl = Just lnurl
                , k1 = Just k1
                , isAuth = True
              }
            , Cmd.none
            )

        LoadLnurlAuth (Err _) ->
            ( model, Cmd.none )

        LoadLnurlAuthResult (Ok result) ->
            ( { model
                | authResult = Just result
              }
            , Cmd.none
            )

        LoadLnurlAuthResult (Err _) ->
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
    { lnurl = Nothing
    , k1 = Nothing
    , authResult = Nothing
    , isAuth = False
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
    if model.isAuth then
        Time.every 1000 (always AuthStatus)

    else
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


viewQrcode : Lnurl -> Element msg
viewQrcode lnurl =
    lnurl
        |> QRCode.fromString
        |> Result.map (QRCode.toSvg [] >> html >> el [ Background.color (rgb255 255 255 255), width fill ])
        |> Result.withDefault (text "Error while encoding to QRCode.")


viewAuthResult : Bool -> Element msg
viewAuthResult result =
    if result then
        text "success"

    else
        text "failure"


viewWelcome : Maybe Bool -> Maybe Lnurl -> Element Msg
viewWelcome result lnurl =
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
            , lnurl
                |> Maybe.map viewQrcode
                |> Maybe.withDefault viewQrcodeLoading
            , result
                |> Maybe.map viewAuthResult
                |> Maybe.withDefault none
            ]


viewResult : Model -> Element Msg
viewResult model =
    let
        children =
            model.links
                |> Maybe.map viewLinkList
                |> Maybe.withDefault [ viewWelcome model.authResult model.lnurl ]
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
