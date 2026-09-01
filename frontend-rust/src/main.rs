use hesocial_frontend::auth::claim_oauth_token_on_boot;
use hesocial_frontend::ui::App;

fn main() {
    claim_oauth_token_on_boot();
    dioxus::launch(App);
}
