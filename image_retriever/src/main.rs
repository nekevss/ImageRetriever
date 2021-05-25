use druid::commands::{OPEN_FILE, SHOW_OPEN_PANEL};
use druid::widget::{Align, Button, Flex, Label, List, Padding, Scroll, TextBox};
use druid::{
    AppDelegate, AppLauncher, Command, Data, DelegateCtx, Env, FileDialogOptions, Handled, Lens,
    Target, WidgetExt, WindowDesc,
};
use im;
use std::env;
use std::path::PathBuf;

mod requests;
use requests::request_runner;

#[derive(Clone, Data, Lens)]
pub struct RetrieverState {
    requests_input: String,
    feedback: im::Vector<String>,
    export_dir: String,
}

impl Default for RetrieverState {
    fn default() -> RetrieverState {
        let directory_buffer: PathBuf = env::current_dir().unwrap_or_default();

        let dir = directory_buffer.to_str().unwrap();

        RetrieverState {
            requests_input: String::new(),
            feedback: im::Vector::new(),
            export_dir: String::from(dir),
        }
    }
}

struct Delly;

impl AppDelegate<RetrieverState> for Delly {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut RetrieverState,
        _env: &Env,
    ) -> Handled {
        if let Some(file_data) = cmd.get(OPEN_FILE) {
            let path_str = file_data.path().to_str().unwrap_or_default();
            data.export_dir = String::from(path_str);

            Handled::Yes
        } else {
            Handled::No
        }
    }
}

fn main() {
    let main_window =
        WindowDesc::new(build_root()).title("Image Retriever! Fetching images since 2021, woof!");

    let init_state = RetrieverState::default();

    AppLauncher::with_window(main_window)
        .delegate(Delly {})
        .launch(init_state)
        .expect("App launch failed")
}

fn build_root() -> impl druid::Widget<RetrieverState> {
    //let's create our top panel, "control panel"
    let prompt = Label::new("Throw some URLs, and I'll fetch them!");

    let select_path = Button::new("Select").on_click(|ctx, _data, _env| {
        let options = FileDialogOptions::new().select_directories();
        ctx.submit_command(SHOW_OPEN_PANEL.with(options.clone()));
    });

    let path_label = Label::new(|data: &RetrieverState, _env: &_| format!("{}", data.export_dir));

    let path_selection = Flex::row()
        .with_child(select_path)
        .with_spacer(4.0)
        .with_child(path_label);

    let panel = Flex::column()
        .with_child(prompt)
        .with_spacer(16.0)
        .with_child(path_selection);

    let input_box = TextBox::multiline()
        .with_placeholder("URLs go here")
        .expand_width()
        .fix_height(224.0)
        .lens(RetrieverState::requests_input);

    let feedback_display = Flex::column()
        .with_child(
            Scroll::new(List::new(|| {
                Align::left(
                    Flex::row()
                        .with_child(Label::new(|item: &String, _env: &_| format!("{}", item))),
                )
            }))
            .vertical()
            .lens(RetrieverState::feedback),
        )
        .fix_height(200.0);

    let fetch_button =
        Button::new("Throw").on_click(|_ctx, data: &mut RetrieverState, _env| request_runner(data));

    let clear_button = Button::new("Clear").on_click(|_ctx, data: &mut RetrieverState, _env| {
        data.requests_input = String::from("");
        data.feedback = im::Vector::new();
    });
    //great, now let's load those buttons into a container
    let button_container = Flex::row()
        .with_child(fetch_button)
        .with_spacer(8.0)
        .with_child(clear_button);

    let layout = Padding::new(
        (16.0, 16.0),
        //create a flex layout with (0.4, 0.4, 0.2)
        Flex::column()
            .with_child(panel)
            .with_spacer(32.0)
            .with_child(input_box)
            .with_spacer(32.0)
            .with_child(feedback_display)
            .with_spacer(32.0)
            .with_child(Align::right(button_container)),
    );

    Align::vertical(druid::UnitPoint::TOP, layout)
}
