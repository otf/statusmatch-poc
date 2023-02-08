module Main exposing (..)

import Browser
import Element exposing (text)
import Element.Input as Input
import Html exposing (Html)


type alias Model =
    ()


type alias Msg =
    ()


update : Msg -> Model -> ( Model, Cmd Msg )
update () model =
    ( model, Cmd.none )


init : () -> ( Model, Cmd Msg )
init () =
    ( (), Cmd.none )


subscriptions : Model -> Sub msg
subscriptions () =
    Sub.none


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }


view : Model -> Html msg
view () =
    Element.layout [] <|
        Input.text []
            { onChange = Debug.todo
            , text = ""
            , placeholder = Nothing
            , label = Input.labelLeft [] <| text "Search a program."
            }
