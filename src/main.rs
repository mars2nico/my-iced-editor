use iced::widget::{button, column, container, horizontal_space, row, text, text_editor};
use iced::{executor, Application, Command, Element, Length, Settings, Theme};

use std::path::{Path, PathBuf};
use std::sync::Arc;

fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    New,
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    Save,
    FileSaved(Result<PathBuf, Error>),
}

impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                path: None,
                content: text_editor::Content::new(),
                error: None,
            },
            Command::perform(load_file(default_file()), Message::FileOpened),
        )
    }

    fn title(&self) -> String {
        String::from("A cool editor!")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
                self.error = None;
                Command::none()
            }
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();
                Command::none()
            }
            Message::Open => Command::perform(pick_file(), Message::FileOpened),
            Message::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with(&content);
                Command::none()
            }
            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
            Message::Save => {
                let text = self.content.text();

                Command::perform(save_file(self.path.clone(), text), Message::FileSaved)
            }
            Message::FileSaved(Ok(path)) => {
                self.path = Some(path);
                Command::none()
            }
            Message::FileSaved(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let controls = row![
            button("New").on_press(Message::New),
            button("Open").on_press(Message::Open),
            button("Save").on_press(Message::Save),
        ];
        let input = text_editor(&self.content).on_edit(Message::Edit);

        let status_bar = {
            let status = if let Some(Error::IOFailed(error)) = &self.error {
                text(error.to_string())
            } else {
                match self.path.as_deref().and_then(Path::to_str) {
                    Some(path) => text(path).size(14),
                    None => text(""),
                }
            };

            let position = {
                let (line, column) = self.content.cursor_position();

                text(format!("{}:{}", line + 1, column + 1))
            };

            row![status, horizontal_space(Length::Fill), position]
        };

        container(column![controls, input, status_bar])
            .padding(10)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let dialog_result = rfd::AsyncFileDialog::new()
        .set_title("Choose a text file...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed);

    load_file(dialog_result?.path().to_owned()).await
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let read_result = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new) // スレッド間で
        .map_err(|error| error.kind())
        .map_err(Error::IOFailed);

    Ok((path, read_result?))
}

async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        let dialog_result = rfd::AsyncFileDialog::new()
            .set_title("Choose a file name ...")
            .save_file()
            .await
            .ok_or(Error::DialogClosed);

        dialog_result?.path().to_owned()
    };

    let write_result = tokio::fs::write(&path, text)
        .await
        .map_err(|error| Error::IOFailed(error.kind()));

    write_result?;
    Ok(path)
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IOFailed(std::io::ErrorKind),
}