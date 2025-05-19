#![allow(non_snake_case)]

use leptos::{
    ev::{KeyboardEvent, keydown},
    prelude::*,
};

// TODO:
// - Use dynamic <For> instead of vec
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
    delete: Callback<usize>,
    edit: Callback<usize>,
}

#[component]
fn App() -> impl IntoView {
    let todos: RwSignal<Vec<Todo>> = RwSignal::new(vec![]);
    let active_todo_key = RwSignal::new(None);
    let current_key = RwSignal::new(0usize);

    let delete_todo = move |key| {
        todos.update(|ts| {
            ts.remove(ts.iter().position(|t| t.key == key).unwrap());
        });
    };

    let edit_todo = move |key| match active_todo_key.get() {
        Some(ak) if ak == key => active_todo_key.set(None),
        None => active_todo_key.set(Some(key)),
        _ => {}
    };

    let add_todo = move || {
        let cur_key_val = current_key.get_untracked();
        todos.update(|t| {
            let new_todo = Todo {
                key: cur_key_val,
                content: RwSignal::new(String::from("todo")),
                delete: Callback::new(move |key| delete_todo(key)),
                edit: Callback::new(move |key| edit_todo(key)),
            };
            t.push(new_todo);
        });
        active_todo_key.set(Some(cur_key_val));
        current_key.set(cur_key_val + 1);
    };

    let _add_todo_shortcut = window_event_listener(keydown, move |key_down: KeyboardEvent| {
        if &key_down.key() == "Enter" && active_todo_key.get_untracked().is_none() {
            key_down.prevent_default();
            add_todo();
        }
    });

    view! {
        <div class="todo-area">
            <button on:click=move |_| add_todo() disabled=move || active_todo_key.get().is_some()>"+"</button>
            {
                move || todos
                    .get()
                    .into_iter()
                    .rev()
                    .map(|t| {
                        let t_is_active = active_todo_key.get().is_some_and(|ak| ak == t.key);
                        view! {
                            <Todo todo=t active=t_is_active />
                        }
                    })
                    .collect_view()
            }
        </div>
    }
}

#[component]
fn Todo(todo: Todo, active: bool) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Textarea>::new();

    Effect::new(move |_| {
        if active {
            let _ = input_ref.get().unwrap().focus();
        }
    });

    view! {
        <div class="todo">
            {
                if active {
                    let finish_editing = move || {
                        let input_text = input_ref.get().unwrap().value().trim().to_string();
                        todo.content.set(input_text);
                        todo.edit.run(todo.key);
                    };

                    let test_enter_text = move |key_press: KeyboardEvent| {
                        if &key_press.key() == "Enter" {
                            if !key_press.shift_key() {
                                finish_editing();
                            }
                        }
                    };

                    view! {
                        <textarea class="todo-text" on:keypress=test_enter_text node_ref=input_ref />
                        <div class="todo-controls">
                            <button on:click=move |_| finish_editing()>"Done"</button>
                            <button on:click=move |_| todo.delete.run(todo.key)>"-"</button>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <pre class="todo-text">{move || format!("{} (#{})", todo.content.get(), todo.key)}</pre>
                        <div class="todo-controls">
                            <button on:click=move |_| todo.edit.run(todo.key)>"Edit"</button>
                            <button on:click=move |_| todo.delete.run(todo.key)>"-"</button>
                        </div>
                    }.into_any()
                }
            }
        </div>
    }
}
