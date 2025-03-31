pub mod format;
pub mod html_renderer;

pub use format::Format;
use html_renderer::{HtmlRenderer, IntoSerializedResponse};

use axum::response::IntoResponse;

pub struct AutoOutputFormat<S, T: IntoSerializedResponse<S, Renderer>, Renderer: HtmlRenderer<S>> {
    data: T,
    format: Format<Renderer::Context>,
    template: &'static str,
    renderer: Renderer,
}

impl<S, T: IntoSerializedResponse<S, Renderer>, Renderer: HtmlRenderer<S>>
    AutoOutputFormat<S, T, Renderer>
{
    pub fn new(
        data: T,
        template: &'static str,
        format: Format<Renderer::Context>,
        renderer: Renderer,
    ) -> Self {
        AutoOutputFormat {
            data,
            format,
            template,
            renderer,
        }
    }

    fn create_json_response(&self) -> axum::response::Response {
        todo!()
        // let mut response = Json(&self.data).into_response();
        // *response.status_mut() = self.status;
        // response
    }
}

impl<S, T: IntoSerializedResponse<S, Cb>, Cb: HtmlRenderer<S>> IntoResponse
    for AutoOutputFormat<S, T, Cb>
{
    fn into_response(self) -> axum::response::Response {
        match self.format {
            Format::Html(ctx) => {
                self.data
                    .into_serialized_response(ctx, self.renderer, self.template)
            }
            Format::Json => self.create_json_response(),
        }
    }
}
