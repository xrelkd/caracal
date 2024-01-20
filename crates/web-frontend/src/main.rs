use gloo_net::http::Request;
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let version = use_state(String::new);
    {
        let version = version.clone();
        use_effect_with((), move |()| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = Request::get("/api/v1/system/version").send().await {
                    let fetched_version = resp.text().await.unwrap_or_default();
                    version.set(fetched_version);
                }
            });
            || ()
        });
    }

    html! {
        <div>
            <p>{ format!("Caracal {}", version.as_str()) }</p>
        </div>
    }
}

fn main() { let _render = yew::Renderer::<App>::new().render(); }
