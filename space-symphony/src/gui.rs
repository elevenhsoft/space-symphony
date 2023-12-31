use crate::{Response, APP_ID};

use anyhow::Error;
use iced_aw::menu::menu_tree::MenuTree;
use std::sync::Arc;

use iced::widget::button::Appearance;
use iced::widget::{button, Column, Container};
use iced::{
    executor, theme, Application, Background, Color, Command, Element, Length, Renderer, Theme,
};
use iced_aw::menu::MenuBar;

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
    Quit,
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
                Buttons::Quit => std::process::exit(0),
            },
            AppMessage::LoggedIn => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let root_1 = MenuTree::with_children(
            button("File").style(theme::Button::Custom(Box::new(TopMenuButton))),
            vec![MenuTree::new(
                button("Quit")
                    .style(theme::Button::Custom(Box::new(TopMenuButton)))
                    .width(Length::Fill)
                    .on_press(AppMessage::Clicked(Buttons::Quit)),
            )],
        );

        let root_2 = MenuTree::with_children(
            button("Spotify").style(theme::Button::Custom(Box::new(TopMenuButton))),
            vec![MenuTree::new(self.login_btn())],
        );

        let menu_bar = MenuBar::new(vec![root_1, root_2]).spacing(10.).padding(10);
        let col = Column::new().push(menu_bar);

        Container::new(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(30)
            .into()
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
                .style(theme::Button::Custom(Box::new(TopMenuButton)))
                .width(Length::Fill)
                .on_press(AppMessage::Clicked(Buttons::Logout))
                .into()
        } else {
            button("login")
                .style(theme::Button::Custom(Box::new(TopMenuButton)))
                .width(Length::Fill)
                .on_press(AppMessage::Clicked(Buttons::Login))
                .into()
        }
    }
}

struct TopMenuButton;

impl button::StyleSheet for TopMenuButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn disabled(&self, style: &Self::Style) -> Appearance {
        let active = self.active(style);

        Appearance {
            shadow_offset: iced::Vector::default(),
            background: active.background.map(|background| match background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
                Background::Gradient(gradient) => Background::Gradient(gradient.mul_alpha(0.5)),
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }

    fn hovered(&self, style: &Self::Style) -> Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.6, 0.6, 0.6))),
            text_color: Color::WHITE,
            ..self.active(style)
        }
    }

    fn pressed(&self, style: &Self::Style) -> Appearance {
        Appearance {
            shadow_offset: iced::Vector::default(),
            ..self.active(style)
        }
    }
}
