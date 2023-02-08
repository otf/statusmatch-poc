module Main exposing (..)

import Browser
import Element exposing (Element, text)
import Element.Input as Input
import Html exposing (Html)
import Http
import Json.Decode exposing (Decoder, list, string)


type alias Model =
    { searchText : String
    , programs : List String
    }


type Msg
    = GotSearchText String
    | LoadPrograms (Result Http.Error (List String))


programsDecoder : Decoder (List String)
programsDecoder =
    list string


fetchPrograms : String -> Cmd Msg
fetchPrograms text =
    Http.get
        { url = "api/programs/search?text=" ++ text
        , expect = Http.expectJson LoadPrograms programsDecoder
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


viewPrograms : List String -> Element Msg
viewPrograms programs =
    let
        children =
            programs
                |> List.map (\p -> text p)
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
        , viewPrograms model.programs
        ]


view : Model -> Html Msg
view model =
    Element.layout [] <|
        viewFrom model
