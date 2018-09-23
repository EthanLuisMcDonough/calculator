use super::gtk;
use super::parser;
use std::cell::RefCell;
use std::char::from_u32;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Button, Entry};

const ROW_LEN: usize = 7;

fn apply_css<T: WidgetExt>(win: &T, bytes: &[u8]) -> Option<Result<(), gtk::Error>> {
    win.get_screen().map(|screen| {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(bytes).map(|_| {
            let priority = gtk::STYLE_PROVIDER_PRIORITY_USER;
            gtk::StyleContext::add_provider_for_screen(&screen, &provider, priority);
        })
    })
}

fn format_ans(ans: f64) -> String {
    if ans.abs() > 1E9 {
        format!("{:E}", ans)
    } else {
        ans.to_string()
    }
}

fn ok_key(c: char) -> bool {
    match c {
        '(' | ')' | '.' | '-' | '+' | '*' | '/' | '^' | 'E' => true,
        _ => c.is_digit(10),
    }
}

#[derive(Clone)]
enum ButtonEvent {
    Inv,
    DegMode,
    Ans,
    Evaluate,
    Clear,
    Del,
}

#[derive(Clone)]
enum ButtonData {
    Special(ButtonEvent),
    Simple,
    Renamed {
        push_in: String,
    },
    Function {
        name: String,
    },
    InvFunction {
        label: String,
        name: String,
        inverted: bool,
        inverted_label: String,
        inverted_name: String,
    },
}

struct CalcButton {
    button: Button,
    data: ButtonData,
}

impl CalcButton {
    fn new(label: &str, data: ButtonData) -> Self {
        let button = Button::new_with_label(label);
        if let Some(ctx) = button.get_style_context() {
            ctx.add_class("calc-button");
        }
        Self { button, data }
    }

    fn new_function(label: &str) -> Self {
        Self::new(label, ButtonData::Function { name: label.into() })
    }

    fn new_simple(label: &str) -> Self {
        Self::new(label, ButtonData::Simple)
    }

    fn new_renamed(true_name: &str, label: &str) -> Self {
        Self::new(
            label,
            ButtonData::Renamed {
                push_in: true_name.into(),
            },
        )
    }

    fn schedule_event(&self, index: usize, state: Rc<RefCell<CalculatorState>>) {
        self.button.connect_clicked(move |b| {
            if let Ok(mut st) = state.try_borrow_mut() {
                if let Some(data) = st.buttons.get(index).map(|button| button.data.clone()) {
                    st.send_btn_message(b, data);
                }
            }
        });
    }
}

pub struct CalculatorState {
    angle_mode: parser::AngleMode,
    prev_ans: Option<f64>,
    buttons: Vec<CalcButton>,
    textarea: Entry,
    mode_index: Option<usize>,
    err_label: gtk::Label,
    clear_next: bool,
}

impl CalculatorState {
    fn new(buttons: Vec<CalcButton>) -> Self {
        let textarea = Entry::new();
        textarea.set_editable(false);
        textarea.set_alignment(1.0);
        textarea.set_placeholder_text(Some("Enter an expression"));
        if let Some(ctx) = textarea.get_style_context() {
            ctx.add_class("calc-textarea");
        }

        let err_label = gtk::Label::new(None);
        err_label.set_line_wrap(true);
        if let Some(ctx) = err_label.get_style_context() {
            ctx.add_class("err-label");
        }

        Self {
            angle_mode: parser::AngleMode::Rad,
            prev_ans: None,
            buttons,
            textarea,
            mode_index: None,
            err_label,
            clear_next: true,
        }
    }

    fn last_ans(&self) -> String {
        self.prev_ans.map(format_ans).unwrap_or_default()
    }

    fn clear(&mut self) {
        self.textarea
            .delete_text(0, self.textarea.get_text_length() as i32);
    }

    fn add_str(&mut self, s: &str) {
        if self.clear_next {
            self.textarea
                .delete_text(0, self.textarea.get_text_length() as i32);
        }
        self.textarea
            .insert_text(s, &mut (self.textarea.get_text_length() as i32));
        self.clear_next = false;
    }

    fn invert(&mut self) {
        for button in self.buttons.iter_mut() {
            if let ButtonData::InvFunction {
                ref mut inverted,
                label,
                inverted_label,
                ..
            } = &mut button.data
            {
                *inverted = !*inverted;
                button
                    .button
                    .set_label(&if *inverted { inverted_label } else { label })
            }
        }
    }

    fn evaluate(&mut self) {
        match parser::eval_math(
            &self.textarea.get_text().unwrap_or_default(),
            self.angle_mode,
        ) {
            Ok(solution) => {
                let fixed = parser::to_fixed(solution, 7);
                self.prev_ans = fixed.into();
                self.textarea.set_text(&format_ans(fixed));
                self.clear_next = true;
            }
            Err(msg) => {
                self.err_label.set_text(msg.as_ref());
            }
        }
    }

    fn backspace(&self, size: u16) {
        self.textarea.delete_text(
            self.textarea
                .get_text_length()
                .checked_sub(size)
                .unwrap_or_default() as i32,
            self.textarea.get_text_length() as i32,
        )
    }

    fn send_btn_message(&mut self, button: &Button, message: ButtonData) {
        use self::ButtonData::*;

        self.err_label.set_text("");

        match message {
            Simple => self.add_str(&button.get_label().unwrap_or_default()),
            Renamed { ref push_in } => self.add_str(push_in),
            Function { name } => self.add_str(&(name + "(")),
            InvFunction {
                name,
                inverted_name,
                inverted,
                ..
            } => self.add_str(&(if inverted { inverted_name } else { name } + "(")),
            Special(ButtonEvent::Inv) => self.invert(),
            Special(ButtonEvent::Ans) => {
                let ans = &self.last_ans();
                self.add_str(ans)
            }
            Special(ButtonEvent::Clear) => self.clear(),
            Special(ButtonEvent::DegMode) => if let Some(index) = self.mode_index {
                if let Some(button) = self.buttons.get(index) {
                    button.button.set_label(&self.angle_mode.to_string());
                    self.angle_mode = !self.angle_mode;
                }
            },
            Special(ButtonEvent::Evaluate) => self.evaluate(),
            Special(ButtonEvent::Del) => self.backspace(1),
        }
    }
}

pub struct Calculator {
    window: gtk::ApplicationWindow,
    state: Rc<RefCell<CalculatorState>>,
}

impl Calculator {
    pub fn new(application: &gtk::Application) -> Self {
        let window = gtk::ApplicationWindow::new(application);

        let header = gtk::HeaderBar::new();
        header.set_title("Scientific Calculator");
        header.set_show_close_button(true);
        header.set_decoration_layout("menu:close");
        window.set_titlebar(&header);

        let mut state = CalculatorState::new(vec![
            // First row
            CalcButton::new("Deg", ButtonData::Special(ButtonEvent::DegMode)),
            CalcButton::new(
                "sin",
                ButtonData::InvFunction {
                    label: "sin".into(),
                    name: "sin".into(),
                    inverted: false,
                    inverted_label: "sin⁻¹".into(),
                    inverted_name: "asin".into(),
                },
            ),
            CalcButton::new_function("round"),
            CalcButton::new_function("ln"),
            CalcButton::new_simple("("),
            CalcButton::new_simple(")"),
            CalcButton::new_renamed("/", "÷"),
            // Second row
            CalcButton::new("Inv", ButtonData::Special(ButtonEvent::Inv)),
            CalcButton::new(
                "cos",
                ButtonData::InvFunction {
                    label: "cos".into(),
                    name: "cos".into(),
                    inverted: false,
                    inverted_label: "cos⁻¹".into(),
                    inverted_name: "acos".into(),
                },
            ),
            CalcButton::new(
                "floor",
                ButtonData::InvFunction {
                    label: "floor".into(),
                    name: "floor".into(),
                    inverted: false,
                    inverted_label: "ceil".into(),
                    inverted_name: "ceil".into(),
                },
            ),
            CalcButton::new_simple("7"),
            CalcButton::new_simple("8"),
            CalcButton::new_simple("9"),
            CalcButton::new_renamed("*", "×"),
            // Third row
            CalcButton::new_renamed("pi", "π"),
            CalcButton::new(
                "tan",
                ButtonData::InvFunction {
                    label: "tan".into(),
                    name: "tan".into(),
                    inverted: false,
                    inverted_label: "tan⁻¹".into(),
                    inverted_name: "atan".into(),
                },
            ),
            CalcButton::new(
                "√",
                ButtonData::Function {
                    name: "sqrt".into(),
                },
            ),
            CalcButton::new_simple("4"),
            CalcButton::new_simple("5"),
            CalcButton::new_simple("6"),
            CalcButton::new_simple("-"),
            // Fourth row
            CalcButton::new_simple("e"),
            CalcButton::new_renamed("E", "EXP"),
            CalcButton::new_function("abs"),
            CalcButton::new_simple("1"),
            CalcButton::new_simple("2"),
            CalcButton::new_simple("3"),
            CalcButton::new_simple("+"),
            // Fifth row
            CalcButton::new("ANS", ButtonData::Special(ButtonEvent::Ans)),
            CalcButton::new_simple("^"),
            CalcButton::new("log10", ButtonData::Function { name: "log".into() }),
            CalcButton::new_simple("0"),
            CalcButton::new_simple("."),
            CalcButton::new_renamed("-", "(-)"),
            CalcButton::new("=", ButtonData::Special(ButtonEvent::Evaluate)),
        ]);

        window.set_title("Calculator");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(555, 350);

        apply_css(&window, include_bytes!("../css/main.css"))
            .expect("ERROR: Could not load window screen")
            .expect("ERROR: Could not load CSS");

        window.connect_delete_event(move |win, _| {
            win.destroy();
            Inhibit(false)
        });

        window.set_resizable(false);

        let grid = gtk::Grid::new();

        let textarea_height = 2usize;
        grid.attach(
            &state.textarea,
            0,
            0,
            ROW_LEN as i32,
            textarea_height as i32,
        );

        let del = CalcButton::new("DEL", ButtonData::Special(ButtonEvent::Del));
        let clear = CalcButton::new("AC", ButtonData::Special(ButtonEvent::Clear));

        grid.attach(
            &del.button,
            ROW_LEN as i32 - 2,
            textarea_height as i32 + 1,
            1,
            1,
        );

        grid.attach(
            &clear.button,
            ROW_LEN as i32 - 1,
            textarea_height as i32 + 1,
            1,
            1,
        );

        grid.attach(
            &state.err_label,
            0,
            textarea_height as i32 + 1,
            ROW_LEN as i32 - 2,
            1,
        );

        grid.set_row_homogeneous(true);
        grid.set_column_homogeneous(true);
        grid.set_column_spacing(5);
        grid.set_row_spacing(5);

        window.add(&grid);

        for (ind, button) in state.buttons.iter().enumerate() {
            grid.attach(
                &button.button,
                (ind % ROW_LEN) as i32,
                (textarea_height + 2 + ind / ROW_LEN) as i32,
                1,
                1,
            );

            if let ButtonData::Special(ButtonEvent::DegMode) = button.data {
                state.mode_index = Some(ind);
            }
        }

        state.buttons.push(del);
        state.buttons.push(clear);

        let calc = Self {
            window,
            state: Rc::new(RefCell::new(state)),
        };

        let keypress_state = calc.state.clone();
        calc.window.connect_key_press_event(move |_, event| {
            if let Ok(mut state) = keypress_state.try_borrow_mut() {
                let keyval = event.get_keyval();
                if let Some(c) = from_u32(keyval).filter(|ch| ok_key(*ch)) {
                    state.add_str(&c.to_string());
                } else if let Some(name) = ::gdk::keyval_name(keyval) {
                    if name == "BackSpace" || name == "Delete" {
                        state.backspace(1);
                    } else if name == "Return" {
                        state.evaluate();
                        return Inhibit(true);
                    }
                }
            }
            Inhibit(false)
        });

        if let Ok(state) = calc.state.try_borrow() {
            for (index, button) in state.buttons.iter().enumerate() {
                button.schedule_event(index, calc.state.clone());
            }
        }

        calc
    }

    pub fn show(&self) {
        self.window.show_all();
    }
}
