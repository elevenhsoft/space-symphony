use crate::{Response, APP_ID};
use std::sync::Arc;

use anyhow::Error;
use iced::widget::{button, Container};
use iced::{executor, Application, Command, Element, Renderer, Theme};
use iced_aw::menu::{MenuBar, MenuTree};
use rspotify::{ClientCredsSpotify, Config, Token, TokenCallback};
use serde::{Deserialize, Serialize};
use spotify_web_api::SpotifyWeb;

#[derive(Clone, Default, Deserialize, Serialize)]
pub enum AuthState {
    Yes,
    #[default]
    No,
}

#[derive(Deserialize, Serialize, Default)]
pub struct SpaceConfig {
    pub response: Option<Response>,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    LoggedIn,
    Clicked(Buttons),
}

#[derive(Debug, Clone)]
pub enum Buttons {
    Login,
    Logout,
}

pub struct SpaceSymphony;

impl Application for SpaceSymphony {
    type Executor = executor::Default;

    type Message = AppMessage;

    type Theme = Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (SpaceSymphony, Command::none())
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn title(&self) -> String {
        String::from("Space Symphony")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMessage::Clicked(btn) => match btn {
                Buttons::Login => Command::perform(Self::authorize(), |_| AppMessage::LoggedIn),
                Buttons::Logout => Command::none(),
            },
            AppMessage::LoggedIn => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let sub_2 = MenuTree::with_children(
            button("Sub Menu 2"),
            vec![
                MenuTree::new(button("item_1")),
                MenuTree::new(button("item_2")),
                MenuTree::new(button("item_3")),
            ],
        );

        let sub_1 = MenuTree::with_children(
            button("Sub Menu 1"),
            vec![
                MenuTree::new(button("item_1")),
                sub_2,
                MenuTree::new(button("item_2")),
                MenuTree::new(button("item_3")),
            ],
        );

        let root_1 = MenuTree::with_children(
            button("File"),
            vec![
                MenuTree::new(button("item_1")),
                MenuTree::new(button("item_2")),
                sub_1,
                MenuTree::new(button("item_3")),
            ],
        );

        let root_2 =
            MenuTree::with_children(button("Spotify"), vec![MenuTree::new(self.login_btn())]);

        let menu_bar = MenuBar::new(vec![root_1, root_2]);

        Container::new(menu_bar).into()
    }
}

impl SpaceSymphony {
    async fn authorize() -> Result<SpotifyWeb, Error> {
        let spotify_web = SpotifyWeb::new();

        let auth_url = spotify_web.auth_url.clone();
        let creds = spotify_web.creds.clone();

        if webbrowser::open(&auth_url).is_ok() {
            println!("Browser opened. Please login on spotify website.");
        }

        let operate_token_fn = |token: Token| {
            let resp = Response {
                access_token: token.access_token,
                expires_in: token.expires_in.into(),
                expires_at: token.expires_at,
                refresh_token: token.refresh_token,
                scopes: token.scopes,
                logged: AuthState::Yes,
            };

            let config = SpaceConfig {
                response: Some(resp),
            };

            confy::store(APP_ID, None, config).expect("storing config");

            Ok(())
        };

        let token_callback = TokenCallback(Box::new(operate_token_fn));
        let config = Config {
            token_callback_fn: Arc::new(Some(token_callback)),
            ..Default::default()
        };

        let spotify = ClientCredsSpotify::with_config(creds, config);
        spotify.request_token().await.unwrap();

        Ok(spotify_web)
    }

    fn login_btn(&self) -> Element<AppMessage> {
        let config: SpaceConfig = confy::load(APP_ID, None).expect("loading config");

        let mut state: bool = false;

        if let Some(resp) = config.response {
            match resp.logged {
                AuthState::Yes => state = true,
                AuthState::No => state = false,
            }
        }

        if state {
            button("logout")
                .on_press(AppMessage::Clicked(Buttons::Logout))
                .into()
        } else {
            button("login")
                .on_press(AppMessage::Clicked(Buttons::Login))
                .into()
        }
    }
}
