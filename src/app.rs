use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = window)]
    fn alert(s: &str);
}

#[function_component(App)]
pub fn app() -> Html {
    let name = use_state(String::new);
    let run = use_state(|| false);
    {
        let run2 = run.clone();
        let run3 = run.clone();
        let name = name;
        use_effect_with_deps(
            move |_| {
                if *run2 {
                    spawn_local(async move {
                        let result = invoke("run", JsValue::NULL).await;
                        name.set(from_value(result).unwrap());
                        alert("Ran the parser :)");
                    });
                    run2.set(false);
                }
                || {}
            },
            run3,
        );
    }
    let onclick = {
        Callback::from(move |_| {
            run.set(true);
        })
    };
    html! {
        <>
        <h1>
        {"We expect your library folder in your Desktop/Parser/Library"}
        </h1>
        <button onclick={onclick}>
            {"Run the parser "}
        </button>
        </>
    }
}
