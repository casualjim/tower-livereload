use crate::routes::components::*;
use axum::{Router, response::IntoResponse, routing::get};
use axum_htmx::HxRequest;
use hypertext::prelude::*;

use crate::app::AppState;

pub fn routes(app: AppState) -> Router<AppState> {
  Router::new().route("/", get(index_page)).with_state(app)
}
fn maybe_document<R: Renderable>(
  HxRequest(is_hx_request): HxRequest,
  children: R,
) -> impl IntoResponse {
  maud! {
    @if is_hx_request {
      (children)
    } @else {
      Document {
        (children)
      }
    }
  }
}

async fn index_page(hx_request: HxRequest) -> impl IntoResponse {
  maybe_document(
    hx_request,
    maud! {
      div class="min-h-screen flex flex-col lg:flex-row overflow-x-hidden" {
        aside
          class="hidden lg:flex lg:w-20 lg:flex-col lg:justify-between lg:border-r lg:border-base-300 lg:bg-base-200 lg:py-4 lg:px-2"
          aria-label="Workspace navigation"
        {
          div class="flex flex-col items-center gap-3" {
            span class="text-[0.55rem] uppercase tracking-[0.4em] text-base-content/50" { "team" }
            button class="btn btn-ghost btn-square text-xl" aria-label="Home workspace" {
              span { "✦" }
            }
            button class="btn btn-ghost btn-square text-lg" aria-label="Shared spaces" {
              span { "⚙" }
            }
            button class="btn btn-ghost btn-square text-lg" aria-label="Boards" {
              span { "≡" }
            }
            button class="btn btn-ghost btn-square text-lg" aria-label="Notifications" {
              span { "!" }
            }
          }
          details class="dropdown dropdown-end dropdown-right px-1" data-profile-dropdown @click.outside="$el.removeAttribute('open')" {
            summary
              id="profile-toggle"
              aria-haspopup="true"
              aria-label="Profile menu"
              data-profile-toggle
              class="btn btn-ghost btn-circle avatar text-base-content"
            {
              "IV"
            }
            ul
              id="profile-menu"
              role="menu"
              aria-label="Profile menu"
              data-profile-menu
              class="dropdown-content menu rounded-box z-21 mt-2 w-44 bg-base-100 p-2 shadow dropdown-bottom"
            {
              li { a role="menuitem" class="active:bg-base-200" { "Profile" } }
              li { a role="menuitem" class="active:bg-base-200" { "Settings" } }
              li { a role="menuitem" class="active:bg-base-200" { "Sign out" } }
            }
          }
        }

        div class="flex flex-1 flex-col" {
          header class="sticky top-0 z-20 border-b border-base-300 bg-base-100/90 px-4 py-3 backdrop-blur" {
            div class="flex items-center justify-between gap-4" {
              div class="flex items-center gap-3" {
                button
                  @click="if (window.innerWidth < 1024 && !leftSidebar) rightSidebar = false; leftSidebar = !leftSidebar"
                  class="btn btn-sm btn-ghost btn-square lg:hidden"
                  aria-label="Toggle left sidebar"
                {
                  i data-lucide="menu" class="w-4 h-4" {}
                }
                button
                  @click="leftSidebar = !leftSidebar"
                  class="hidden lg:inline-flex btn btn-sm btn-ghost btn-square"
                  aria-label="Toggle left sidebar"
                {
                  i x-show="leftSidebar" data-lucide="panel-left-close" class="w-4 h-4" {}
                  i x-show="!leftSidebar" data-lucide="panel-left-open" class="w-4 h-4" {}
                }
                div class="text-xl font-semibold leading-tight" { "{{project-name}}" }
                span class="badge badge-outline badge-primary" { "Workspace" }
              }
              div class="flex items-center gap-2" {
                button
                  @click="cycleTheme()"
                  class="btn btn-sm btn-ghost btn-circle"
                  aria-label="Toggle theme"
                {
                  ph-circle-half x-show="theme === 'system'" size="20" {}
                  ph-sun x-show="theme === 'light'" size="20" {}
                  ph-moon x-show="theme === 'dark'" size="20" {}
                }
                button
                  @click="if (window.innerWidth < 1024 && !rightSidebar) leftSidebar = false; rightSidebar = !rightSidebar"
                  class="btn btn-sm btn-ghost btn-square lg:hidden"
                  aria-label="Toggle right sidebar"
                {
                  i data-lucide="menu" class="w-4 h-4" {}
                }
                button
                  @click="rightSidebar = !rightSidebar"
                  class="hidden lg:inline-flex btn btn-sm btn-ghost btn-square"
                  aria-label="Toggle right sidebar"
                {
                  i x-show="rightSidebar" data-lucide="panel-right-close" class="w-4 h-4" {}
                  i x-show="!rightSidebar" data-lucide="panel-right-open" class="w-4 h-4" {}
                }
              }
            }
          }

          div class="flex flex-1 overflow-hidden" {
            div
              x-show="leftSidebar || rightSidebar"
              "x-transition:enter"="transition-opacity ease-out duration-300"
              "x-transition:enter-start"="opacity-0"
              "x-transition:enter-end"="opacity-100"
              "x-transition:leave"="transition-opacity ease-in duration-300"
              "x-transition:leave-start"="opacity-100"
              "x-transition:leave-end"="opacity-0"
              class="lg:hidden fixed inset-0 bg-black/50 z-20"
            {}

            aside
              aria-label="Left sidebar"
              x-show="leftSidebar"
              @click.outside="if (window.innerWidth < 1024) leftSidebar = false"
              "x-transition:enter"="transition-all ease-out duration-300"
              "x-transition:enter-start"="-ml-80 opacity-0"
              "x-transition:enter-end"="ml-0 opacity-100"
              "x-transition:leave"="transition-all ease-in duration-300"
              "x-transition:leave-start"="ml-0 opacity-100"
              "x-transition:leave-end"="-ml-80 opacity-0"
              :style="`width: ${leftWidth}px`"
              class="border-r border-base-300 bg-base-200 overflow-y-auto shrink-0 fixed lg:relative left-0 top-0 bottom-0 z-30 lg:z-auto"
            {
              div class="p-4 space-y-6" {
                section class="card bg-base-100 shadow rounded-lg" {
                  div class="card-body" {
                    h3 class="text-lg font-semibold" { "Pinned docs" }
                    div class="mt-4 flex flex-col gap-3" {
                      article class="flex items-center justify-between rounded-box border border-base-200 bg-base-100 p-3" {
                        div {
                          p class="font-semibold" { "Product brief" }
                          p class="text-sm text-base-content/60" { "Strategy • 3 readers" }
                        }
                        button class="btn btn-xs btn-outline" { "Open" }
                      }
                      article class="flex items-center justify-between rounded-box border border-base-200 bg-base-100 p-3" {
                        div {
                          p class="font-semibold" { "Launch retro" }
                          p class="text-sm text-base-content/60" { "Retro • 5 readers" }
                        }
                        button class="btn btn-xs btn-outline" { "Open" }
                      }
                    }
                  }
                }

                section class="card bg-base-100 shadow rounded-lg" {
                  div class="card-body" {
                    h3 class="text-lg font-semibold" { "Navigation" }
                    div class="menu bg-base-100 rounded-box" {
                      li { a { "Dashboard" } }
                      li { a { "Projects" } }
                      li { a { "Team" } }
                      li { a { "Settings" } }
                    }
                  }
                }
              }
              div
                data-resize-handle="left"
                @mousedown="startResize('left', $event)"
                class="absolute top-0 right-0 bottom-0 w-1 cursor-col-resize hover:bg-primary/50 transition-colors"
              {}
            }

            main aria-label="Main content area" class="flex-1 overflow-y-auto px-4 py-6 pb-28 lg:pb-6 space-y-6" {
              section class="card bg-base-100 shadow rounded-lg" {
                div class="card-body" {
                  div class="flex items-center justify-between" {
                    h2 class="text-lg font-semibold" { "Daily pulse" }
                    span class="text-xs text-base-content/70" { "Updated just now" }
                  }
                  p class="text-sm text-base-content/70" {
                    "Keep an eye on the conversations, docs, and quick wins that keep the workspace moving."
                  }
                  div class="mt-4 grid grid-cols-2 gap-4 sm:grid-cols-4" {
                    div class="stat rounded-box border border-base-200 bg-base-100 p-3" {
                      div class="stat-title text-[0.7rem] text-base-content/60" { "Active docs" }
                      div class="stat-value text-lg" { 12 }
                    }
                    div class="stat rounded-box border border-base-200 bg-base-100 p-3" {
                      div class="stat-title text-[0.7rem] text-base-content/60" { "Live chats" }
                      div class="stat-value text-lg" { 4 }
                    }
                    div class="stat rounded-box border border-base-200 bg-base-100 p-3" {
                      div class="stat-title text-[0.7rem] text-base-content/60" { "Focus rooms" }
                      div class="stat-value text-lg" { 3 }
                    }
                    div class="stat rounded-box border border-base-200 bg-base-100 p-3" {
                      div class="stat-title text-[0.7rem] text-base-content/60" { "Unread" }
                      div class="stat-value text-lg" { 21 }
                    }
                  }
                }
              }

              section class="card bg-base-100 shadow rounded-lg" {
                div class="card-body" {
                  h3 class="text-lg font-semibold" { "Recent Activity" }
                  div class="mt-4 space-y-4" {
                    article class="rounded-box border border-base-200 bg-base-100 p-3" {
                      p class="text-sm text-base-content/70" { "You joined the #strategy channel." }
                    }
                    article class="rounded-box border border-base-200 bg-base-100 p-3" {
                      p class="text-sm text-base-content/70" { "New task added to Sprint 12 board." }
                    }
                    article class="rounded-box border border-base-200 bg-base-100 p-3" {
                      p class="text-sm text-base-content/70" { "Design review at 3:30 PM in Focus Room." }
                    }
                  }
                }
              }

              section class="card bg-base-100 shadow rounded-lg" {
                div class="card-body" {
                  h3 class="text-lg font-semibold" { "Content Section" }
                  p class="text-sm text-base-content/70" {
                    (r#"This is the main content area. It scrolls independently from the sidebars.
                    Add your content here."#)
                  }
                }
              }
            }

            aside
              aria-label="Right sidebar"
              x-show="rightSidebar"
              @click.outside="if (window.innerWidth < 1024) rightSidebar = false"
              "x-transition:enter"="transition-all ease-out duration-300"
              "x-transition:enter-start"="-mr-80 opacity-0"
              "x-transition:enter-end"="mr-0 opacity-100"
              "x-transition:leave"="transition-all ease-in duration-300"
              "x-transition:leave-start"="mr-0 opacity-100"
              "x-transition:leave-end"="-mr-80 opacity-0"
              :style="`width: ${rightWidth}px`"
              class="border-l border-base-300 bg-base-200 overflow-y-auto shrink-0 fixed lg:relative right-0 top-0 bottom-0 z-30 lg:z-auto"
            {
              div
                data-resize-handle="right"
                @mousedown="startResize('right', $event)"
                class="absolute top-0 left-0 bottom-0 w-1 cursor-col-resize hover:bg-primary/50 transition-colors"
              {}
              div class="p-4 space-y-6" {
                section class="card bg-base-100 shadow rounded-lg" {
                  div class="card-body" {
                    h3 class="text-lg font-semibold" { "Activity feed" }
                    div class="mt-4 space-y-4" {
                      article class="rounded-box border border-base-200 bg-base-100 p-3" {
                        p class="text-sm text-base-content/70" { "You joined the #strategy channel." }
                      }
                      article class="rounded-box border border-base-200 bg-base-100 p-3" {
                        p class="text-sm text-base-content/70" { "New task added to Sprint 12 board." }
                      }
                      article class="rounded-box border border-base-200 bg-base-100 p-3" {
                        p class="text-sm text-base-content/70" { "Design review at 3:30 PM in Focus Room." }
                      }
                    }
                  }
                }

                section class="card bg-base-100 shadow rounded-lg" {
                  div class="card-body" {
                    h3 class="text-lg font-semibold" { "Quick Actions" }
                    div class="space-y-2 mt-4" {
                      button class="btn btn-block btn-sm" { "New Document" }
                      button class="btn btn-block btn-sm" { "Start Chat" }
                      button class="btn btn-block btn-sm" { "Schedule Meeting" }
                    }
                  }
                }
              }
            }
          }
        }
      }
      nav class="lg:hidden fixed bottom-0 left-0 right-0 z-50 flex items-center justify-between border-t border-base-300 bg-base-200 px-4 py-2" aria-label="Mobile workspace navigation" {
        div class="flex items-center gap-4" {
          button class="btn btn-ghost btn-sm" aria-label="Home workspace" {
            span class="text-2xl" { "✦" }
          }
          button class="btn btn-ghost btn-sm" aria-label="Shared spaces" {
            span class="text-xl" { "⚙" }
          }
          button class="btn btn-ghost btn-sm" aria-label="Boards" {
            span class="text-xl" { "≡" }
          }
          button class="btn btn-ghost btn-sm" aria-label="Notifications" {
            span class="text-xl" { "!" }
          }
        }
        details class="dropdown dropdown-top relative" data-profile-dropdown-mobile @click.outside="$el.removeAttribute('open')" {
          summary
            aria-haspopup="true"
            aria-label="Profile menu"
            class="btn btn-ghost btn-circle avatar text-base-content text-sm"
          {
            "IV"
          }
          ul
            role="menu"
            aria-label="Profile menu"
            class="dropdown-content menu rounded-box z-21 mb-2 w-max bg-base-100 px-3 py-2 shadow absolute right-0"
          {
            li { a role="menuitem" class="active:bg-base-200" { "Profile" } }
            li { a role="menuitem" class="active:bg-base-200" { "Settings" } }
            li { a role="menuitem" class="active:bg-base-200" { "Sign out" } }
          }
        }
      }
    },
  )
}
