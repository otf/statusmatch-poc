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


type alias Model =
    { searchText : String
    , programs : List Program_
    }


type Msg
    = GotSearchText String
    | LoadPrograms (Result Http.Error (List Program_))


programDecoder : D.Decoder Program_
programDecoder =
    D.map2 Program_ (D.field "id" D.int) (D.field "name" D.string)


programListDecoder : D.Decoder (List Program_)
programListDecoder =
    D.list programDecoder


fetchPrograms : String -> Cmd Msg
fetchPrograms text =
    Http.get
        { url = "api/programs/search?text=" ++ text
        , expect = Http.expectJson LoadPrograms programListDecoder
        }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotSearchText text ->
            ( { model
                | searchText = text
              }
            , fetchPrograms text
            )

        LoadPrograms (Ok programs) ->
            ( { model
                | programs = programs
              }
            , Cmd.none
            )

        LoadPrograms (Err _) ->
            ( model, Cmd.none )


initModel : Model
initModel =
    { searchText = "", programs = [] }


init : () -> ( Model, Cmd Msg )
init () =
    ( initModel, Cmd.none )


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
    text <|
        String.fromInt program.id
            ++ ":"
            ++ program.name


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


viewFrom : Model -> Element Msg
viewFrom model =
    Element.column
        []
        [ Input.search []
            { onChange = GotSearchText
            , text = model.searchText
            , placeholder = Nothing
            , label = Input.labelLeft [] <| text "Search a program."
            }
        , viewProgramList model.programs
        ]


view : Model -> Html Msg
view model =
    Element.layout [] <|
        viewFrom model
