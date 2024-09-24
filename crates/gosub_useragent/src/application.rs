use std::collections::HashMap;
use std::sync::mpsc;

use anyhow::anyhow;
use url::Url;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::WindowId;

use gosub_render_backend::{NodeDesc, RenderBackend};
use gosub_render_backend::layout::{Layouter, LayoutTree};
use gosub_renderer::draw::SceneDrawer;
use gosub_shared::traits::css3::CssSystem;
use gosub_shared::traits::document::Document;
use gosub_shared::traits::html5::Html5Parser;
use gosub_shared::types::Result;

use crate::window::Window;

#[derive(Debug, Default)]
pub struct WindowOptions {
    #[cfg(target_arch = "wasm32")] pub id: String,
}

impl WindowOptions {
    #[cfg(target_arch = "wasm32")]
    pub fn with_id(id: String) -> Self {
        Self { id }
    }
}


pub struct Application<
    'a,
    D: SceneDrawer<B, L, LT, Doc, C>,
    B: RenderBackend,
    L: Layouter,
    LT: LayoutTree<L>,
    Doc: Document<C>,
    C: CssSystem,
    P: Html5Parser<C, Document = Doc>,
> {
    open_windows: Vec<(Vec<Url>, WindowOptions)>, // Vec of Windows, each with a Vec of URLs, representing tabs
    windows: HashMap<WindowId, Window<'a, D, B, L, LT, Doc, C>>,
    backend: B,
    layouter: L,
    proxy: Option<EventLoopProxy<CustomEvent>>,
    event_loop: Option<EventLoop<CustomEvent>>,
    debug: bool,
    _marker: std::marker::PhantomData<&'a P>,
}

impl<
        'a,
        D: SceneDrawer<B, L, LT, Doc, C>,
        B: RenderBackend,
        L: Layouter,
        LT: LayoutTree<L>,
        Doc: Document<C>,
        C: CssSystem,
        P: Html5Parser<C, Document = Doc>,
    > ApplicationHandler<CustomEvent> for Application<'a, D, B, L, LT, Doc, C, P>
{
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        for window in self.windows.values_mut() {
            if let Err(e) = window.resumed(&mut self.backend) {
                eprintln!("Error resuming window: {e:?}");
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: CustomEvent) {
        match event {
            CustomEvent::OpenWindow(url, id) => {
                let mut window = match Window::new(
                    event_loop,
                    &mut self.backend,
                    self.layouter.clone(),
                    url,
                    self.debug,
                    id,
                ) {
                    Ok(window) => window,
                    Err(e) => {
                        eprintln!("Error opening window: {e:?}");

                        if self.windows.is_empty() {
                            event_loop.exit();
                        }
                        return;
                    }
                };

                if let Err(e) = window.resumed(&mut self.backend) {
                    eprintln!("Error resuming window: {e:?}");
                    return;
                }
                self.windows.insert(window.id(), window);
            }
            CustomEvent::CloseWindow(id) => {
                self.windows.remove(&id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            CustomEvent::OpenInitial => {
                for (urls, opts) in self.open_windows.drain(..) {
                    let mut window = match Window::new::<P>(
                        event_loop,
                        &mut self.backend,
                        self.layouter.clone(),
                        urls[0].clone(),
                        self.debug,
                        opts,
                    ) {
                        Ok(window) => window,
                        Err(e) => {
                            eprintln!("Error opening window: {e:?}");
                            if self.windows.is_empty() {
                                event_loop.exit();
                            }
                            return;
                        }
                    };

                    if let Err(e) = window.resumed(&mut self.backend) {
                        eprintln!("Error resuming window: {e:?}");
                        if self.windows.is_empty() {
                            event_loop.exit();
                        }
                        return;
                    }

                    self.windows.insert(window.id(), window);
                }
            }
            CustomEvent::Select(id) => {
                if let Some(window) = self.windows.values_mut().next() {
                    window.select_element(LT::NodeId::from(id));
                    window.request_redraw();
                }
            }
            CustomEvent::SendNodes(sender) => {
                for window in self.windows.values_mut() {
                    window.send_nodes(sender.clone());
                }
            }

            CustomEvent::Unselect => {
                if let Some(window) = self.windows.values_mut().next() {
                    window.unselect_element();
                    window.request_redraw();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            if let Err(e) = window.event(event_loop, &mut self.backend, event) {
                eprintln!("Error handling window event: {e:?}");
            };
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        for window in self.windows.values_mut() {
            window.suspended(event_loop, &mut self.backend);
        }
    }
}

impl<
        'a,
        D: SceneDrawer<B, L, LT, Doc, C>,
        B: RenderBackend,
        L: Layouter,
        LT: LayoutTree<L>,
        Doc: Document<C>,
        C: CssSystem,
        P: Html5Parser<C, Document = Doc>,
    > Application<'a, D, B, L, LT, Doc, C, P>
{
    pub fn new(backend: B, layouter: L, debug: bool) -> Self {
        Self {
            windows: HashMap::new(),
            backend,
            layouter,
            proxy: None,
            event_loop: None,
            open_windows: Vec::new(),
            debug,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn initial_tab(&mut self, url: Url, opts: WindowOptions) {
        self.open_windows.push((vec![url], opts));
    }

    pub fn initial(&mut self, mut windows: Vec<(Vec<Url>, WindowOptions)>) {
        self.open_windows.append(&mut windows);
    }

    pub fn add_window(&mut self, window: Window<'a, D, B, L, LT, Doc, C>) {
        self.windows.insert(window.window.id(), window);
    }

    pub fn open_window(&mut self, url: Url, opts: WindowOptions) {
        if let Some(proxy) = &self.proxy {
            let _ = proxy.send_event(CustomEvent::OpenWindow(url, opts));
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        let event_loop = EventLoop::with_user_event().build()?;

        self.proxy = Some(event_loop.create_proxy());
        self.event_loop = Some(event_loop);

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        if self.event_loop.is_none() {
            self.initialize()?;
        }

        let event_loop = self.event_loop.take().ok_or(anyhow!("No event loop; unreachable!"))?;

        let proxy = self.proxy()?;

        proxy
            .send_event(CustomEvent::OpenInitial)
            .map_err(|e| anyhow!(e.to_string()))?;

        event_loop.run_app(self)?;

        Ok(())
    }

    pub fn proxy(&mut self) -> Result<EventLoopProxy<CustomEvent>> {
        if self.proxy.is_none() {
            self.initialize()?;
        }

        self.proxy.clone().ok_or(anyhow!("No proxy; unreachable!"))
    }

    pub fn close_window(&mut self, id: WindowId) {
        if let Some(proxy) = &self.proxy {
            let _ = proxy.send_event(CustomEvent::CloseWindow(id));
        }
    }
}

#[derive(Debug)]
pub enum CustomEvent {
    OpenWindow(Url, WindowOptions),
    CloseWindow(WindowId),
    OpenInitial,
    Select(u64),
    SendNodes(mpsc::Sender<NodeDesc>),
    Unselect,
}
