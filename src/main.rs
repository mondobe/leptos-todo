#![allow(non_snake_case)]

use leptos::prelude::*;

// TODO:
// - Fancy CSS
// - Automatically make new item active
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
    let add_todo = move |_| {
        todos.update(|t| {
            let new_todo = Todo {
                key: current_key.get_untracked(),
                content: RwSignal::new(String::from("todo")),
                delete: Callback::new(move |key| delete_todo(key)),
                edit: Callback::new(move |key| edit_todo(key)),
            };
            t.push(new_todo);
        });
        current_key.set(current_key.get_untracked() + 1);
    };
    view! {
        <button on:click=add_todo disabled=move || active_todo_key.get().is_some()>"+"</button>
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
    }
}

#[component]
fn Todo(todo: Todo, active: bool) -> impl IntoView {
    view! {
        <div>
            {
                if active {
                    let input_ref = NodeRef::<leptos::html::Input>::new();
                    let finish_editing = move |_| {
                        let input_text = input_ref.get().unwrap().value();
                        todo.content.set(input_text);
                        todo.edit.run(todo.key);
                    };
                    view! {
                        <input type="text" node_ref=input_ref />
                        <button on:click=finish_editing>"Done"</button>
                    }.into_any()
                } else {
                    view! {
                        <p>{move || format!("{} (#{})", todo.content.get(), todo.key)}</p>
                        <button on:click=move |_| todo.edit.run(todo.key)>"Edit"</button>
                    }.into_any()
                }
            }
            <button on:click=move |_| todo.delete.run(todo.key)>"-"</button>
        </div>
    }
}
