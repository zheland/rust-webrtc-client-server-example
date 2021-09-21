use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlElement, Navigator, Node, Window};

fn window() -> Window {
    web_sys::window().unwrap()
}

fn document() -> Document {
    window().document().unwrap()
}

pub fn navigator() -> Navigator {
    window().navigator()
}

pub fn body() -> HtmlElement {
    document().body().unwrap()
}

pub trait ElementExt {
    fn add_child<T: JsCast>(&self, name: &str) -> T;
    fn add_text(&self, text: &str);
    fn remove(&self);
}

impl ElementExt for Element {
    fn add_child<T: JsCast>(&self, name: &str) -> T {
        let node = document().create_element(name).unwrap();
        let _: Node = self.append_child(&node).unwrap();
        node.dyn_into().unwrap()
    }

    fn add_text(&self, text: &str) {
        let node = document().create_text_node(text);
        let _: Node = self.append_child(&node).unwrap();
    }

    fn remove(&self) {
        let _: Node = self.parent_element().unwrap().remove_child(self).unwrap();
    }
}
