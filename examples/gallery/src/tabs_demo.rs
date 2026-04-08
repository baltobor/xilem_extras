//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Tabs widget demo.

use masonry::layout::AsUnit;
use xilem::masonry::peniko::Color;
use xilem::style::Style;
use xilem::view::{button, flex_col, flex_row, label, portal};
use xilem::WidgetView;

use xilem_extras::{TabBar, TabBarColors, TabItem};

use crate::app_model::AppModel;

const TEXT_COLOR: Color = Color::from_rgb8(220, 218, 214);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 156, 150);
const BG_TAB_ACTIVE: Color = Color::from_rgb8(55, 53, 50);
const BG_TAB_INACTIVE: Color = Color::from_rgb8(45, 43, 40);
const BG_TAB_BAR: Color = Color::from_rgb8(38, 36, 34);

/// Tab content for the demo.
pub struct DemoTab {
    pub title: String,
    pub content: String,
    pub dirty: bool,
}

impl DemoTab {
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            dirty: false,
        }
    }
}

impl TabItem for DemoTab {
    fn title(&self) -> &str {
        &self.title
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

/// Creates the demo tabs with literary content.
pub fn create_demo_tabs() -> Vec<DemoTab> {
    vec![
        DemoTab::new(
            "Lorem Ipsum",
            r#"Lorem ipsum dolor sit amet, consectetur adipiscing elit,
Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
Ut enim ad minim veniam, quis nostrud exercitation,
Ullamco laboris nisi ut aliquip ex ea commodo consequat.

Duis aute irure dolor in reprehenderit in voluptate,
Velit esse cillum dolore eu fugiat nulla pariatur.
Excepteur sint occaecat cupidatat non proident,
Sunt in culpa qui officia deserunt mollit anim id est laborum.

Curabitur pretium tincidunt lacus, nulla gravida orci a odio,
Nullam varius, turpis et commodo pharetra,
Est eros bibendum elit, nec luctus magna,
Felis sollicitudin mauris."#,
        ),
        DemoTab::new(
            "The Raven",
            r#"Once upon a midnight dreary, while I pondered, weak and weary,
Over many a quaint and curious volume of forgotten lore—
While I nodded, nearly napping, suddenly there came a tapping,
As of some one gently rapping, rapping at my chamber door.
"'Tis some visitor," I muttered, "tapping at my chamber door—
            Only this and nothing more."

Ah, distinctly I remember it was in the bleak December;
And each separate dying ember wrought its ghost upon the floor.
Eagerly I wished the morrow;—vainly I had sought to borrow
From my books surcease of sorrow—sorrow for the lost Lenore—
For the rare and radiant maiden whom the angels name Lenore—
            Nameless here for evermore.

— Edgar Allan Poe, 1845"#,
        ),
        DemoTab::new(
            "Joy of Cycling",
            r#"There is something magical about cycling.
You can smell nature and feel the wind tugging at your clothes.
With every turn of the pedals, you experience the freedom
to move through space and time – under your own steam.

On a bike, you experience things you would miss in a car.
From there, the road is just grey with a white line.
The scent of the plants. Flowers, the forest, the birds.
You feel the path beneath the wheels.


Every hill you conquer brings joy; every descent is a thrill.

Cycling connects us with our surroundings and our community;
you can take a deep breath and glide through the countryside.
You wave to your neighbours, whistle a tune. And lose yourself
in thought. In the moment. In the now. You discover hidden paths...

It needs no petrol and still takes you far away. The bicycle.
It is pure, unadulterated freedom.
"#,
        ),
    ]
}

pub fn tabs_demo(model: &mut AppModel) -> impl WidgetView<AppModel> + use<'_> {
    let active_idx = model.demo_active_tab;

    // Configure colors to match our theme
    let colors = TabBarColors {
        active_bg: BG_TAB_ACTIVE,
        inactive_bg: BG_TAB_INACTIVE,
        bar_bg: BG_TAB_BAR,
        text: TEXT_COLOR,
        text_secondary: TEXT_SECONDARY,
    };

    // Build the tab bar using the library component
    let show_nav = model.demo_show_tab_nav;
    let tab_bar = TabBar::new(&model.demo_tabs, active_idx)
        .colors(colors)
        .show_nav_buttons(show_nav)
        .on_select(|model: &mut AppModel, idx| {
            model.demo_active_tab = idx;
        })
        .on_close(|model: &mut AppModel, idx| {
            model.close_demo_tab(idx);
        })
        .build();

    // Content panel
    let content = if let Some(tab) = model.demo_tabs.get(active_idx) {
        label(tab.content.clone())
            .text_size(14.0)
            .color(TEXT_COLOR)
            .boxed()
    } else {
        label("No tabs open")
            .text_size(14.0)
            .color(TEXT_SECONDARY)
            .boxed()
    };

    flex_col((
        label("Tabs Demo")
            .text_size(16.0)
            .weight(xilem::FontWeight::BOLD)
            .color(TEXT_COLOR),
        label("Click tabs to switch, X to close, arrows to navigate")
            .text_size(12.0)
            .color(TEXT_SECONDARY),
        tab_bar,
        portal(flex_col((content,)).padding(16.0)),
        // Actions
        flex_row((
            button(label("Mark Dirty"), move |model: &mut AppModel| {
                if let Some(tab) = model.demo_tabs.get_mut(model.demo_active_tab) {
                    tab.dirty = !tab.dirty;
                }
            }),
            button(label("Add Tab"), |model: &mut AppModel| {
                let n = model.demo_tabs.len() + 1;
                model.demo_tabs.push(DemoTab::new(
                    format!("New Tab {}", n),
                    "A fresh new tab with empty content.".to_string(),
                ));
                model.demo_active_tab = model.demo_tabs.len() - 1;
            }),
            button(label("Toggle Nav Buttons"), |model: &mut AppModel| {
                model.demo_show_tab_nav = !model.demo_show_tab_nav;
            }),
        ))
        .gap(8.px()),
    ))
    .gap(8.px())
    .padding(16.0)
}
