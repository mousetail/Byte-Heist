mod auto_output_format;

use auto_output_format::html_renderer::IntoSerializedResponse;
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::handler::Handler;
use axum::response::{IntoResponse, Response};
use std::pin::Pin;

pub use auto_output_format::html_renderer::CustomResponseMetadata;
pub use auto_output_format::html_renderer::HtmlRenderer;

pub struct OutputWrapperFactory<M> {
    pub renderer: M,
}

impl<M: Clone> OutputWrapperFactory<M> {
    pub fn handler<T>(&self, template_name: &'static str, handler: T) -> AutoOutputWrapper<T, M> {
        AutoOutputWrapper {
            handler,
            template: template_name,
            renderer: self.renderer.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AutoOutputWrapper<T, M> {
    handler: T,
    template: &'static str,
    renderer: M,
}

#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!([], T1);
        $name!([T1], T2);
        $name!([T1, T2], T3);
        $name!([T1, T2, T3], T4);
        $name!([T1, T2, T3, T4], T5);
        $name!([T1, T2, T3, T4, T5], T6);
        $name!([T1, T2, T3, T4, T5, T6], T7);
        $name!([T1, T2, T3, T4, T5, T6, T7], T8);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], T14);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], T15);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], T16);
    };
}

macro_rules! impl_handler {
    (
        [$($ty:ident),*], $last:ident
    ) => {
        #[allow(non_snake_case, unused_mut)]
        impl<F, Fut, S, Res, M, $($ty,)* $last, Hal> Handler<(M, $($ty,)* $last,), S> for AutoOutputWrapper<F, Hal>
        where
            F: FnOnce($($ty,)* $last,) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = Result<Res, <Hal as HtmlRenderer<S>>::Err>> + Send,
            S: Send + Sync + 'static,
            Res: IntoSerializedResponse<S, Hal> + Send + Sync + 'static,
            $( $ty: FromRequestParts<S> + Send, )*
            $last: FromRequest<S, M>,
            Hal: HtmlRenderer<S> + Clone + 'static + Send + Sync,
            <<Hal as HtmlRenderer<S>>::Context as FromRequestParts<S>>::Rejection: std::fmt::Debug
        {
            type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

            fn call(self, req: Request, state: S) -> Self::Future {
                let (mut parts, body) = req.into_parts();

                Box::pin(async move {
                    let output_format = auto_output_format::Format::from_request_parts(&mut parts, &state).await.unwrap();

                    $(
                        let $ty = match $ty::from_request_parts(&mut parts, &state).await {
                            Ok(value) => value,
                            Err(rejection) => return rejection.into_response(),
                        };
                    )*

                    let req = Request::from_parts(parts, body);

                    let $last = match $last::from_request(req, &state).await {
                        Ok(value) => value,
                        Err(rejection) => return rejection.into_response(),
                    };

                    let data = (self.handler)($($ty,)* $last,).await;

                    match data {
                        Ok(data) => {
                            auto_output_format::AutoOutputFormat::new(data, self.template, output_format, self.renderer).into_response()
                        }
                        Err(e) => self.renderer.render_error(e)
                    }
                })
            }
        }
    };
}

all_the_tuples!(impl_handler);
