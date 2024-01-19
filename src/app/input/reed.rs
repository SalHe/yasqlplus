use std::{
    borrow::Cow,
    cell::RefCell,
    sync::{Arc, RwLock},
};

use nu_ansi_term::Style;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, DefaultHinter, DefaultPrompt, DefaultPromptSegment,
    Emacs, FileBackedHistory, KeyCode, KeyModifiers, Prompt, Reedline, ReedlineEvent, ReedlineMenu,
    Signal,
};

use crate::{
    app::{
        completer::YspCompleter,
        context::{self, Context},
        highlight::YspHightligter,
        validate::YspValidator,
    },
    command::{parse_command, Command, ParseError},
};

use super::{Input, InputError, INDICATOR, INDICATOR_NORMAL};

pub struct Reed {
    context: Arc<RwLock<Context>>,
    rl: RefCell<Reedline>,
    rl_line: RefCell<Reedline>,
}

impl Reed {
    pub fn new(context: Arc<RwLock<Context>>, history_file: String) -> Result<Self, InputError> {
        // TODO completion
        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Tab,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::Menu("completion_menu".to_string()),
                ReedlineEvent::MenuNext,
            ]),
        );
        let edit_mode = Box::new(Emacs::new(keybindings));

        let rl_line = Reedline::create();
        let rl = Reedline::create()
            .with_validator(Box::new(YspValidator))
            .with_highlighter(Box::new(YspHightligter::new()))
            .with_completer(Box::new(YspCompleter::new(context.clone())))
            .with_menu(ReedlineMenu::EngineCompleter(Box::new(
                ColumnarMenu::default().with_name("completion_menu"),
            )))
            .with_hinter(Box::new(
                DefaultHinter::default().with_style(Style::new().bold()),
            ))
            .with_history(Box::new(FileBackedHistory::with_file(
                4096,
                history_file.clone().into(),
            )?))
            .with_edit_mode(edit_mode);
        Ok(Self {
            context,
            rl: RefCell::new(rl),
            rl_line: RefCell::new(rl_line),
        })
    }
}

impl Input for Reed {
    fn get_command(&self) -> Result<Option<(Command, String)>, InputError> {
        let mut rl = self.rl.borrow_mut();
        let prompt = self.context.read().unwrap().get_prompt();
        let prompt = MyPrompt {
            prompt: prompt.clone(),
            base_prompt: DefaultPrompt::new(
                DefaultPromptSegment::Basic(prompt.to_string()),
                DefaultPromptSegment::CurrentDateTime,
            ),
        };
        match rl.read_line(&prompt)? {
            Signal::Success(input) => match parse_command(&input) {
                Ok(command) => Ok(Some((command, input))),
                Err(ParseError::Empty) => Ok(None),
                Err(err) => Err(err.into()),
            },
            Signal::CtrlC => Err(InputError::Cancelled),
            Signal::CtrlD => Err(InputError::Eof),
        }
    }

    fn line(&self, prompt: &str) -> Result<String, InputError> {
        let mut rl = self.rl_line.borrow_mut();
        match rl.read_line(&DefaultPrompt::new(
            DefaultPromptSegment::Basic(prompt.to_string()),
            DefaultPromptSegment::CurrentDateTime,
        ))? {
            Signal::Success(input) => Ok(input),
            Signal::CtrlC => Err(InputError::Cancelled),
            Signal::CtrlD => Err(InputError::Eof),
        }
    }
}

struct MyPrompt {
    prompt: context::Prompt,
    base_prompt: DefaultPrompt,
}

impl Prompt for MyPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<str> {
        self.base_prompt.render_prompt_left()
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<str> {
        self.base_prompt.render_prompt_right()
    }

    fn render_prompt_indicator(
        &self,
        prompt_mode: reedline::PromptEditMode,
    ) -> std::borrow::Cow<str> {
        match prompt_mode {
            reedline::PromptEditMode::Default => Cow::Borrowed(INDICATOR),
            reedline::PromptEditMode::Emacs => Cow::Borrowed(INDICATOR),
            reedline::PromptEditMode::Vi(v) => match v {
                reedline::PromptViMode::Normal => Cow::Borrowed(INDICATOR_NORMAL),
                reedline::PromptViMode::Insert => Cow::Borrowed(INDICATOR),
            },
            reedline::PromptEditMode::Custom(_) => Cow::Borrowed(INDICATOR),
        }
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
        match &self.base_prompt.left_prompt {
            DefaultPromptSegment::Basic(s) => {
                (".".repeat(s.chars().count()) + &(" ".repeat(INDICATOR.len()))).into()
            }
            _ => "".into(),
        }
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        self.base_prompt
            .render_prompt_history_search_indicator(history_search)
    }

    fn get_indicator_color(&self) -> reedline::Color {
        self.get_prompt_color()
    }

    fn get_prompt_color(&self) -> reedline::Color {
        match self.prompt {
            context::Prompt::Ready => reedline::Color::White,
            context::Prompt::Connected(_) => reedline::Color::Green,
        }
    }
}
