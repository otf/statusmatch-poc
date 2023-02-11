module Main exposing (..)

import Browser
import Element exposing (Element, text)
import Element.Input as Input
import Html exposing (Html)
import Http
import Json.Decode as D


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


type alias Model =
    { searchText : String
    , programs : Maybe (List Program_)
    , statuses : Maybe (List Status)
    , selectedProgram : Maybe Program_
    , selectedStatus : Maybe Status
    , links : Maybe (List Link)
    }


type Msg
    = GotSearchText String
    | LoadPrograms (Result Http.Error (List Program_))
    | LoadStatuses (Result Http.Error (List Status))
    | LoadLinks (Result Http.Error (List Link))
    | ChooseProgram Program_
    | ChooseStatus Status
    | Diagnose


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


fetchPrograms : String -> Cmd Msg
fetchPrograms text =
    Http.get
        { url = "api/programs/search?text=" ++ text
        , expect = Http.expectJson LoadPrograms programListDecoder
        }


fetchStatuses : Program_ -> Cmd Msg
fetchStatuses program =
    Http.get
        { url = "api/statuses/find?program_id=" ++ String.fromInt program.id
        , expect = Http.expectJson LoadStatuses statusListDecoder
        }


fetchLinks : Program_ -> Status -> Cmd Msg
fetchLinks program status =
    let
        strProgramId =
            String.fromInt program.id

        strStatusLevel =
            String.fromInt status.level

        url =
            "api/links/diagnose?program_id=" ++ strProgramId ++ "&status=" ++ strStatusLevel
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
              }
            , fetchStatuses program
            )

        ChooseStatus status ->
            ( { model
                | selectedStatus = Just status
              }
            , Cmd.none
            )

        Diagnose ->
            let
                fetchCmd =
                    Maybe.map2 fetchLinks model.selectedProgram model.selectedStatus
                        |> Maybe.withDefault Cmd.none
            in
            ( model, fetchCmd )


initialModel : Model
initialModel =
    { searchText = ""
    , programs = Nothing
    , statuses = Nothing
    , selectedProgram = Nothing
    , selectedStatus = Nothing
    , links = Nothing
    }


init : () -> ( Model, Cmd Msg )
init () =
    ( initialModel, Cmd.none )


subscriptions : Model -> Sub msg
subscriptions model =
    Sub.none


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }


viewProgram : Program_ -> Element Msg
viewProgram program =
    let
        asString =
            String.fromInt program.id
                ++ ":"
                ++ program.name
    in
    Input.button []
        { onPress = Just <| ChooseProgram program
        , label = text asString
        }


viewProgramList : List Program_ -> Element Msg
viewProgramList programs =
    let
        children =
            programs
                |> List.map viewProgram
    in
    Element.column
        []
        children


viewStatus : Status -> Input.Option Status msg
viewStatus status =
    let
        asString =
            String.fromInt status.level
                ++ ":"
                ++ status.name
    in
    Input.option status <| text asString


viewStatusList : Maybe Status -> List Status -> Element Msg
viewStatusList selected statuses =
    Input.radioRow
        []
        { onChange = ChooseStatus
        , selected = selected
        , label = Input.labelAbove [] <| text "Select your status"
        , options = statuses |> List.map viewStatus
        }


viewForm : Model -> Element Msg
viewForm model =
    Element.column
        []
        [ Input.search []
            { onChange = GotSearchText
            , text = model.searchText
            , placeholder = Nothing
            , label = Input.labelLeft [] <| text "Search a program."
            }
        , model.programs
            |> Maybe.map viewProgramList
            |> Maybe.withDefault Element.none
        , model.statuses
            |> Maybe.map (viewStatusList model.selectedStatus)
            |> Maybe.withDefault Element.none
        ]


viewLink : Link -> Element Msg
viewLink link =
    text (link.program ++ "(" ++ link.status ++ ")")


viewLinkList : List Link -> List (Element Msg)
viewLinkList links =
    links
        |> List.map viewLink


viewResult : Model -> Element Msg
viewResult model =
    let
        children =
            model.links
                |> Maybe.map viewLinkList
                |> Maybe.withDefault []
    in
    Element.column
        []
        children


viewDiagnoseButton : Element Msg
viewDiagnoseButton =
    Input.button []
        { onPress = Just <| Diagnose
        , label = text "Diagnose"
        }


view : Model -> Html Msg
view model =
    Element.layout [] <|
        Element.column
            []
            [ viewForm model
            , viewDiagnoseButton
            , viewResult model
            ]
