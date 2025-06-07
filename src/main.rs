#![allow(non_snake_case)]

use leptos::{
    ev::{KeyboardEvent, beforeunload},
    html::{Button, Input, Textarea},
    prelude::*,
    server::codee::string::FromToStringCodec,
};
use leptos_use::{storage::use_local_storage, use_event_listener};
use serde::{Deserialize, Serialize};
use web_sys::{
    Blob, HtmlLinkElement, Url,
    wasm_bindgen::{JsValue, prelude::Closure},
};

fn main() {
    // Enable enhanced errors from panics in the browser console
    console_error_panic_hook::set_once();

    // Attach app to webpage body
    leptos::mount::mount_to_body(|| {
        view! {
            <h1>"My To-do List"</h1>
            <App />
            <p class="disclaimer">"By using the website, you agree to allow to-do items to be
        stored locally on your device."</p>
        }
    })
}

// Holds all state necessary for storing and displaying a to-do item
#[derive(Debug, Clone)]
struct Todo {
    key: usize, // Unique identifier for each to-do item
    content: RwSignal<String>,

    // Callbacks for actions when clicking on parts of the to-do item
    edit: Callback<(TodoArea, usize)>,
    complete: Callback<usize>,
    restore: Callback<usize>,
    delete: Callback<usize>,
}

// Only used for importing/exporting to-do items to/from files or local storage
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
    // Stores state for active and completed to-do items
    let active_todos: RwSignal<Vec<Todo>> = RwSignal::new(vec![]);
    let completed_todos: RwSignal<Vec<Todo>> = RwSignal::new(vec![]);

    // HTML element references for accessing certain fixed elements
    let import_ref: NodeRef<Input> = NodeRef::new();
    let add_ref: NodeRef<Button> = NodeRef::new();

    // Key of the to-do item currently being edited, if one exists
    let active_todo_key: RwSignal<Option<(TodoArea, usize)>> = RwSignal::new(None);

    // Key of the next to-do item
    let current_key = RwSignal::new(0usize);

    // Whether to expand the box of completed to-do items
    let show_completed = RwSignal::new(false);

    // If editing a to-do item, finish editing it. Otherwise, start editing a new one
    let edit_todo = move |area, key| match active_todo_key.get() {
        Some((_, ak)) if ak == key => active_todo_key.set(None),
        None => active_todo_key.set(Some((area, key))),
        _ => {}
    };

    // Move the given to-do item from the active list to the completed list
    let complete_todo = move |key| {
        active_todos.update(|ts| {
            let removed_todo = ts.remove(ts.iter().position(|t| t.key == key).unwrap());
            completed_todos.update(|ts| {
                // If the completed list is empty, automatically hide it next
                // time we complete a to-do item
                if ts.is_empty() {
                    show_completed.set(false);
                }
                ts.push(removed_todo);
            })
        });
    };

    // Remove the given to-do item from the completed list
    let delete_todo = move |key| {
        completed_todos.update(|ts| {
            ts.remove(ts.iter().position(|t| t.key == key).unwrap());
        });
    };

    // Restore the given to-do item from the completed list to the active list
    let restore_todo = move |key| {
        completed_todos.update(|ts| {
            let removed_todo = ts.remove(ts.iter().position(|t| t.key == key).unwrap());
            active_todos.update(|ts| {
                ts.push(removed_todo);
            })
        });
    };

    // Create a new to-do item and push it to the active list
    let push_todo = move |list: &mut Vec<Todo>| {
        let cur_key_val = current_key.get_untracked();
        let new_todo = Todo {
            key: cur_key_val,
            content: RwSignal::new(String::new()),
            edit: Callback::new(move |(area, key)| edit_todo(area, key)),
            complete: Callback::new(move |key| complete_todo(key)),
            delete: Callback::new(move |key| delete_todo(key)),
            restore: Callback::new(move |key| restore_todo(key)),
        };
        let cur_len = list.len();
        list.push(new_todo);
        // Move to the next key
        current_key.set(cur_key_val + 1);
        // Return the current key to enable editing the new to-do item immediately
        (cur_len, cur_key_val)
    };

    // Add the given to-do item to the active list and start editing it
    let add_todo = move || {
        active_todos.update(|t| {
            let (_, new_todo_key) = push_todo(t);
            active_todo_key.set(Some((TodoArea::Active, new_todo_key)));
        });
    };

    // Import a set of active and completed to-do items from a JSON string
    let import_from_string = move |text: String| {
        let Ok(saved): Result<SavedTodos, _> = serde_json::from_str(&text) else {
            return;
        };
        active_todos.update(|ts| {
            ts.clear();
            for t in saved.active {
                let (i, _) = push_todo(ts);
                ts[i].content.set(t);
            }
        });
        completed_todos.update(|ts| {
            ts.clear();
            for t in saved.completed {
                let (i, _) = push_todo(ts);
                ts[i].content.set(t);
            }
        });
    };

    // Import a set of active and completed to-do items from a JavaScript value
    let import_from_js_value = Closure::new(move |js_text: JsValue| {
        let text = js_text.as_string().unwrap();
        import_from_string(text);
    });

    // Called when clicking on the "Import" button, starts the process of importing
    // items from a JSON file
    let import = move || {
        let input_elem: web_sys::HtmlInputElement = import_ref.get().unwrap();
        let files = input_elem.files().unwrap();
        let file = files.item(0).unwrap();
        let _file_text_promise = file.text().then(&import_from_js_value);
    };

    // Serialize all active and completed to-do items to JSON
    let todos_json = move || {
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

        serde_json::to_string(&saved).unwrap()
    };

    // Export active and completed to-do items to the user's hard drive
    let export = move || {
        // Export to-do items as JSON string to a blob
        let saved_js = JsValue::from_str(&todos_json());
        let blob = Blob::new_with_str_sequence(&web_sys::js_sys::Array::of1(&saved_js)).unwrap();
        let blob_url = Url::create_object_url_with_blob(&blob).unwrap();

        // Create a link to download the blob
        let download_element: JsValue = document().create_element("a").unwrap().into();
        let download_element: HtmlLinkElement = download_element.into();

        // Set the filename with a timestamp of the download
        let timestamp = chrono::Local::now().format("%y-%m%d-%H%M");
        download_element
            .set_attribute("download", &format!("todo-{timestamp}.json"))
            .unwrap();

        // Add the download link to the webpage and click it automatically
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

    // Local storage to autosave to-do items
    let (todos, set_todos, _) = use_local_storage::<String, FromToStringCodec>("saved-todos");

    Effect::new(move |_| {
        import_from_string(todos.get());
    });

    let _ = use_event_listener(window(), beforeunload, move |_| set_todos.set(todos_json()));

    // Autofocus "+" button
    Effect::new(move |_| {
        if active_todo_key.get().is_none() {
            let _ = add_ref.get().unwrap().focus();
        }
    });

    view! {
        // Button to import to-do items from JSON file
        <label class="import">
            <input type="file" accept="text/json" class="import" on:change=move |_| import() node_ref=import_ref />
            Import
        </label>

        // Button to export to-do items to JSON file
        <button on:click=move |_| export()>Export</button>

        // Box of active to-do items
        <div class="todo-area active-todo-area">
            // Button to add new to-do item
            <button on:click=move |_| add_todo() disabled=move || active_todo_key.get().is_some() node_ref=add_ref>"+"</button>

            // List of active items
            {
                move || active_todos
                    .get()
                    .into_iter()
                    .rev()
                    .map(|t| {
                        // Check if item is being edited
                        let t_is_active = active_todo_key.get().is_some_and(|(aa, ak)| matches!(aa, TodoArea::Active) && ak == t.key);
                        view! {
                            <Todo todo=t active=t_is_active area=TodoArea::Active />
                        }
                    })
                    .collect_view()
            }
        </div>

        {
            // Only show completed to-do items if at least one item has been completed
            move || if completed_todos.get().is_empty() {
                ().into_any()
            } else {
                view! {
                    // Box of completed to-do items
                    <div class="todo-area completed-todo-area">
                        // Button to show or hide completed items
                        <button on:click=move |_| {show_completed.set(!show_completed.get());}>{
                            move || if show_completed.get() {
                                "Hide Completed"
                            } else {
                                "Show Completed"
                            }
                        }</button>

                        // Only show list of items if the toggle is enabled
                        <Show when=move || show_completed.get() fallback=move || ()>
                            {
                                move || completed_todos
                                    .get()
                                    .into_iter()
                                    .rev()
                                    .map(|t| {
                                        // Check whether item is currently being edited
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
    // HTML element reference to the editing text area (if it exists)
    let input_ref: NodeRef<Textarea> = NodeRef::new();

    // If being edited, autofocus the text area
    Effect::new(move |_| {
        if active {
            let _ = input_ref.get().unwrap().focus();
        }
    });

    // Finish editing the to-do item (pass new value to App and set as inactive)
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
                    // Finish editing if Enter key pressed (if Shift+Enter, create a new line instead)
                    let test_enter_text = move |key_press: KeyboardEvent| {
                        if &key_press.key() == "Enter" && !key_press.shift_key() {
                            finish_editing();
                        }
                    };

                    // Show text area if being edited
                    view! {
                        <textarea class="todo-text" on:keypress=test_enter_text prop:value=todo.content node_ref=input_ref />
                    }.into_any()
                } else {
                    // Show only pre-formatted text if not being edited
                    view! {
                        <pre class="todo-text">{todo.content}</pre>
                    }.into_any()
                }
            }

            <div class="todo-controls">
                {
                    if active {
                        // Button to finish editing item
                        view !{
                            <button on:click=move |_| finish_editing()>"Done"</button>
                        }.into_any()
                    } else {
                        // Button to edit item
                        view !{
                            <button on:click=move |_| todo.edit.run((area, todo.key))>"Edit"</button>
                        }.into_any()
                    }
                }
                {
                    match area {
                        TodoArea::Active => {
                            // If in the "active to-do items" list, show button
                            // to complete item
                            let on_complete = move |_| {
                                finish_editing();
                                todo.complete.run(todo.key);
                            };
                            view !{
                                <button on:click=on_complete>"-"</button>
                            }.into_any()
                        }
                        TodoArea::Completed => {
                            // If in the completed items list, show buttons to
                            // restore or delete item
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
