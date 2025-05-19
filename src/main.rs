#![allow(non_snake_case)]

use leptos::{
    ev::KeyboardEvent,
    html::{Button, Input, Textarea},
    leptos_dom::logging::console_log,
    prelude::*,
};
use serde::{Deserialize, Serialize, Serializer};
use web_sys::{
    Blob, HtmlLinkElement, Url,
    wasm_bindgen::{JsCast, JsValue},
};

// TODO:
// - JSON Import/Export

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| {
        view! {
            <h1>"My To-do List"</h1>
            <App />
        }
    })
}

#[derive(Debug, Clone)]
struct Todo {
    key: usize,
    content: RwSignal<String>,
    edit: Callback<(TodoArea, usize)>,
    complete: Callback<usize>,
    restore: Callback<usize>,
    delete: Callback<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedTodos {
    active: Vec<String>,
    completed: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum TodoArea {
    Active,
    Completed,
}

#[component]
fn App() -> impl IntoView {
    let active_todos: RwSignal<Vec<Todo>> = RwSignal::new(vec![]);
    let completed_todos: RwSignal<Vec<Todo>> = RwSignal::new(vec![]);
    let import_ref: NodeRef<Input> = NodeRef::new();
    let add_ref: NodeRef<Button> = NodeRef::new();
    let active_todo_key: RwSignal<Option<(TodoArea, usize)>> = RwSignal::new(None);
    let current_key = RwSignal::new(0usize);
    let show_completed = RwSignal::new(false);

    let edit_todo = move |area, key| match active_todo_key.get() {
        Some((_, ak)) if ak == key => active_todo_key.set(None),
        None => active_todo_key.set(Some((area, key))),
        _ => {}
    };

    let complete_todo = move |key| {
        active_todos.update(|ts| {
            let removed_todo = ts.remove(ts.iter().position(|t| t.key == key).unwrap());
            completed_todos.update(|ts| {
                if ts.is_empty() {
                    show_completed.set(false);
                }
                ts.push(removed_todo);
            })
        });
    };

    let delete_todo = move |key| {
        completed_todos.update(|ts| {
            ts.remove(ts.iter().position(|t| t.key == key).unwrap());
        });
    };

    let restore_todo = move |key| {
        completed_todos.update(|ts| {
            let removed_todo = ts.remove(ts.iter().position(|t| t.key == key).unwrap());
            active_todos.update(|ts| {
                ts.push(removed_todo);
            })
        });
    };

    let add_todo = move || {
        let cur_key_val = current_key.get_untracked();
        active_todos.update(|t| {
            let new_todo = Todo {
                key: cur_key_val,
                content: RwSignal::new(String::new()),
                edit: Callback::new(move |(area, key)| edit_todo(area, key)),
                complete: Callback::new(move |key| complete_todo(key)),
                delete: Callback::new(move |key| delete_todo(key)),
                restore: Callback::new(move |key| restore_todo(key)),
            };
            t.push(new_todo);
        });
        active_todo_key.set(Some((TodoArea::Active, cur_key_val)));
        current_key.set(cur_key_val + 1);
    };

    let import = move || {
        let input_elem: web_sys::HtmlInputElement = import_ref.get().unwrap();
        let files = input_elem.files();
    };

    let export = move || {
        let saved = SavedTodos {
            active: active_todos
                .get()
                .into_iter()
                .map(|t| t.content.get())
                .collect(),
            completed: completed_todos
                .get()
                .into_iter()
                .map(|t| t.content.get())
                .collect(),
        };

        let saved_js = JsValue::from_str(&serde_json::to_string(&saved).unwrap());
        let blob = Blob::new_with_str_sequence(&web_sys::js_sys::Array::of1(&saved_js)).unwrap();
        let blob_url = Url::create_object_url_with_blob(&blob).unwrap();

        let download_element: JsValue = document().create_element("a").unwrap().into();
        let download_element: HtmlLinkElement = download_element.into();

        download_element
            .set_attribute("download", "todo.json")
            .unwrap();
        download_element.set_attribute("href", &blob_url).unwrap();
        document()
            .body()
            .unwrap()
            .append_child(&download_element)
            .unwrap();
        download_element.click();
        document()
            .body()
            .unwrap()
            .remove_child(&download_element)
            .unwrap();
    };

    Effect::new(move |_| {
        if active_todo_key.get().is_none() {
            let _ = add_ref.get().unwrap().focus();
        }
    });

    view! {
        <input type="file" accept="text/json" on:change=move |_| import() node_ref=import_ref />
        <button on:click=move |_| export()>Export</button>
        <div class="todo-area active-todo-area">
            <button on:click=move |_| add_todo() disabled=move || active_todo_key.get().is_some() node_ref=add_ref>"+"</button>
            {
                move || active_todos
                    .get()
                    .into_iter()
                    .rev()
                    .map(|t| {
                        let t_is_active = active_todo_key.get().is_some_and(|(aa, ak)| matches!(aa, TodoArea::Active) && ak == t.key);
                        view! {
                            <Todo todo=t active=t_is_active area=TodoArea::Active />
                        }
                    })
                    .collect_view()
            }
        </div>

        {
            move || if completed_todos.get().is_empty() {
                view!{}.into_any()
            } else {
                view! {
                    <div class="todo-area completed-todo-area">
                        <button on:click=move |_| {show_completed.set(!show_completed.get());}>{
                            move || if show_completed.get() {
                                "Hide Completed"
                            } else {
                                "Show Completed"
                            }
                        }</button>
                        <Show when=move || show_completed.get() fallback=move || view!{}>
                            {
                                move || completed_todos
                                    .get()
                                    .into_iter()
                                    .rev()
                                    .map(|t| {
                                        let t_is_active = active_todo_key.get().is_some_and(|(aa, ak)| matches!(aa, TodoArea::Completed) && ak == t.key);
                                        view! {
                                            <Todo todo=t active=t_is_active area=TodoArea::Completed />
                                        }
                                    })
                                    .collect_view()
                            }
                        </Show>
                    </div>
                }.into_any()
            }
        }
    }
}

#[component]
fn Todo(todo: Todo, active: bool, area: TodoArea) -> impl IntoView {
    let input_ref: NodeRef<Textarea> = NodeRef::new();

    Effect::new(move |_| {
        if active {
            let _ = input_ref.get().unwrap().focus();
        }
    });

    let finish_editing = move || {
        if active {
            let input_text = input_ref.get().unwrap().value().trim().to_string();
            todo.content.set(input_text);
            todo.edit.run((area, todo.key));
        }
    };

    view! {
        <div class="todo">
            {
                if active {
                    let test_enter_text = move |key_press: KeyboardEvent| {
                        if &key_press.key() == "Enter" {
                            if !key_press.shift_key() {
                                finish_editing();
                            }
                        }
                    };

                    view! {
                        <textarea class="todo-text" on:keypress=test_enter_text prop:value=todo.content node_ref=input_ref />
                    }.into_any()
                } else {
                    view! {
                        <pre class="todo-text">{todo.content}</pre>
                    }.into_any()
                }
            }

            <div class="todo-controls">
                {
                    if active {
                        view !{
                            <button on:click=move |_| finish_editing()>"Done"</button>
                        }.into_any()
                    } else {
                        view !{
                            <button on:click=move |_| todo.edit.run((area, todo.key))>"Edit"</button>
                        }.into_any()
                    }
                }
                {
                    match area {
                        TodoArea::Active => {
                            let on_complete = move |_| {
                                finish_editing();
                                todo.complete.run(todo.key);
                            };
                            view !{
                                <button on:click=on_complete>"-"</button>
                            }.into_any()
                        }
                        TodoArea::Completed => {
                            let on_restore = move |_| {
                                finish_editing();
                                todo.restore.run(todo.key);
                            };
                            let on_delete = move |_| {
                                finish_editing();
                                todo.delete.run(todo.key);
                            };
                            view !{
                                <button on:click=on_restore>"^"</button>
                                <button on:click=on_delete>"Delete"</button>
                            }.into_any()
                        },
                    }
                }
            </div>
        </div>
    }
}
