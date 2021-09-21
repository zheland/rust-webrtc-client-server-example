use core::cell::RefCell;

use async_std::sync::Arc;
use js_sys::Function;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::convert::FromWasmAbi;

pub type ClosureCell1<T1> = RefCell<Option<Closure<dyn FnMut(T1)>>>;

pub trait WeakCallback<T, F> {
    fn with_weak_callback(arc: &Arc<T>, callback: F) -> Self;
}

impl<T, F, T1> WeakCallback<T, F> for Closure<dyn FnMut(T1)>
where
    T: 'static,
    F: 'static + FnMut(&Arc<T>, T1),
    T1: 'static + FromWasmAbi,
{
    fn with_weak_callback(arc: &Arc<T>, mut callback: F) -> Self {
        let weak = Arc::downgrade(arc);
        let handler: Box<dyn FnMut(T1)> = Box::new(move |arg1| {
            if let Some(arc) = weak.upgrade() {
                callback(&arc, arg1)
            }
        });
        Closure::wrap(handler)
    }
}

// Saves the closure to a cell and uses its reference to set web-sys callback.
// Before the cell is dropped you need to manually clear the web-sys callback.
pub fn init_weak_callback<T, F, G, S, T1>(
    arc: &Arc<T>,
    callback: F,
    cell: &ClosureCell1<T1>,
    setter: G,
    setter_self: S,
) where
    T: 'static,
    F: 'static + FnMut(&Arc<T>, T1),
    G: FnOnce(S, Option<&Function>),
    T1: 'static + FromWasmAbi,
{
    use wasm_bindgen::JsCast;

    let closure = Closure::with_weak_callback(arc, callback);
    setter(setter_self, Some(closure.as_ref().unchecked_ref()));
    let prev = cell.replace(Some(closure));
    assert!(prev.is_none());
}
