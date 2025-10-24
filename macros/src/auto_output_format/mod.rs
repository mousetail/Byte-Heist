pub mod html_renderer;

use html_renderer::{HtmlRenderer, IntoSerializedResponse};

use axum::response::IntoResponse;

pub struct AutoOutputFormat<S, T: IntoSerializedResponse<S, Renderer>, Renderer: HtmlRenderer<S>> {
    data: T,
    context: Renderer::Context,
    template: &'static str,
    renderer: Renderer,
}

impl<S, T: IntoSerializedResponse<S, Renderer>, Renderer: HtmlRenderer<S>>
    AutoOutputFormat<S, T, Renderer>
{
    pub fn new(
        data: T,
        template: &'static str,
        format: Renderer::Context,
        renderer: Renderer,
    ) -> Self {
        AutoOutputFormat {
            data,
            context: format,
            template,
            renderer,
        }
    }
}

impl<S, T: IntoSerializedResponse<S, Cb>, Cb: HtmlRenderer<S>> IntoResponse
    for AutoOutputFormat<S, T, Cb>
{
    fn into_response(self) -> axum::response::Response {
        self.data
            .into_serialized_response(self.context, self.renderer, self.template)
    }
}
