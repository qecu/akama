pub mod add_account;
pub mod add_contact;

use iced::widget::*;
use iced::*;
use widget::{opaque, stack};

fn modal<'a, Message>(
    base: Option<impl Into<Element<'a, Message>>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let mut stack = stack![];
    stack = stack.push_maybe(base);
    stack =
        //base.into(),
        stack.push(
            opaque(
                mouse_area(center(opaque(content)).style(|_theme| {
                    container::Style {
                        background: Some(
                            Color {
                                a: 0.8,
                                ..Color::BLACK
                            }
                            .into(),
                        ),
                        ..container::Style::default()
                    }
                }))
                .on_press(on_blur)
            )
        );

    stack.into()
}
