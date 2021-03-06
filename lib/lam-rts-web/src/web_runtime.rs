use super::refs::*;
use anyhow::Error;
use lam_emu::{
    Coordinator, List, Literal, Program, Ref, RunFuel, Runtime, Scheduler, SchedulerManager,
    Stepper, Value, MFA,
};
use log::*;
use num_bigint::BigInt;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WebSchedulerManager {
    initial_args: Value,
    stepper: Stepper,
}

impl WebSchedulerManager {
    pub fn new(
        initial_args: Value,
        reduction_count: u64,
        step_count: u32,
        program: &Program,
    ) -> WebSchedulerManager {
        let stepper = Scheduler::new(0, reduction_count, program.clone())
            .boot(initial_args.clone())
            .clone()
            .stepper(RunFuel::Bounded(step_count));
        WebSchedulerManager {
            initial_args,
            stepper,
        }
    }
}

impl SchedulerManager for WebSchedulerManager {
    fn setup(&mut self, _scheduler_count: u32, _program: &Program) -> Result<(), Error> {
        info!("Booting up scheduler...");
        Ok(())
    }

    fn run(&self, _coordinator: &Coordinator) -> Result<(), Error> {
        debug!("Stepping scheduler...");
        let runtime = Box::new(WebRuntime::new());
        self.stepper.step(runtime)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WebRuntime {
    window: web_sys::Window,
    document: web_sys::Document,
}

impl WebRuntime {
    pub fn new() -> WebRuntime {
        let window = web_sys::window().expect("Could not get window");
        let document = window.document().expect("Could not get document");
        WebRuntime { window, document }
    }
}

impl Runtime for WebRuntime {
    fn execute(&mut self, mfa: &MFA, args: &[Literal]) -> Literal {
        let MFA {
            module,
            function,
            arity,
        } = mfa;
        debug!("MFA: {:?}", mfa);
        match (module.as_str(), function.as_str(), arity) {
            ("io", "format", 2) => {
                let str = match args[1].clone() {
                    Literal::List(List::Cons(boxed_int, _)) => match *boxed_int {
                        Literal::Integer(bi) => bi.to_string(),
                        _ => format!("{:?}", args),
                    },
                    _ => format!("{:?}", args),
                };
                console::log_1(&str.into());
                Literal::Atom("ok".to_string())
            }
            ("erlang", "-", 2) => {
                let a: BigInt = args[0].clone().into();
                let b: BigInt = args[1].clone().into();
                Literal::Integer(a - b)
            }
            ("erlang", "+", 2) => {
                let a: BigInt = args[0].clone().into();
                let b: BigInt = args[1].clone().into();
                Literal::Integer(a + b)
            }
            ("erlang", "integer_to_binary", 1) => {
                let a: BigInt = args[0].clone().into();
                Literal::Binary(a.to_string())
            }
            ("date", "now", 0) => Literal::Float(js_sys::Date::now().into()),
            ("dom_document", "create_element", 1) => {
                let tag: String = args[0].clone().into();
                let element = self.document.create_element(tag.as_str()).unwrap();
                let webref: WebRef = element.into();
                Literal::Ref(webref.into())
            }
            ("dom_document", "get_element_by_id", 1) => {
                let id: String = args[0].clone().into();
                let element = self.document.get_element_by_id(id.as_str()).unwrap();
                let webref: WebRef = element.into();
                Literal::Ref(webref.into())
            }
            ("dom_element", "append_child", 2) => {
                let parent: Ref = args[0].clone().into();
                let parent_ref: WebRef = WebRef { ref_: parent };
                let parent_element: Rc<RefCell<Element>> = parent_ref.into();

                let child: Ref = args[1].clone().into();
                let child_ref: WebRef = WebRef { ref_: child };
                let child_element: Rc<RefCell<Element>> = child_ref.into();

                parent_element
                    .borrow_mut()
                    .append_child(&child_element.borrow())
                    .unwrap();

                Literal::Atom("ok".to_string())
            }
            ("dom_element", "set_inner_text", 2) => {
                let el: Ref = args[0].clone().into();
                let text: String = args[1].clone().into();
                let webref: WebRef = WebRef { ref_: el };
                let element: Rc<RefCell<Element>> = webref.into();

                element.borrow_mut().set_inner_html(&text);

                Literal::Atom("ok".to_string())
            }
            (_, _, _) => panic!("MFA unimplemented!"),
        }
    }

    fn r#yield(&self) {
        ();
    }
}
