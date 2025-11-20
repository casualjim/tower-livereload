use hypertext::{define_elements, prelude::*};

define_elements! {
  ph_circle_half {
    size
    weight
    color
    mirrored
  }

  ph_sun {
    size
    weight
    color
    mirrored
  }

  ph_moon {
    size
    weight
    color
    mirrored
  }
}

#[component]
pub fn document<R: Renderable>(children: &R) -> impl Renderable {
  maud! {
    !DOCTYPE
    html lang="en" {
      head {
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        title { "{{project-name}}" }
        link rel="stylesheet" href="/static/css/main.css";
        script src="/static/js/main.js" defer=true type="module" {}
      }
      body class="min-h-screen bg-base-100 text-base-content overflow-x-hidden" x-data="layoutState" x-init="init()" @mousemove.window="doResize($event)" @mouseup.window="stopResize()" {
        (children)
      }
    }
  }
}
