use crate::{
    editor_settings::SeedQuerySetting,
    persistence::{SerialieditsyncEditor, DB},
    scroll::ScrollAnchor,
    Anchor, Autoscroll, Editor, EditorEvent, EditorSettings, ExcerptId, ExcerptRange, MultiBuffer,
    MultiBufferSnapshot, NavigationData, SearchWithinRange, ToPoint as _,
};
use anyhow::{anyhow, Context as _, Result};
use collections::HashSet;
use file_icons::FileIcons;
use futures::future::try_join_all;
use git::repository::GitFileStatus;
use gpui::{
    point, AnyElement, AppContext, AsyncWindowContext, Context, Entity, EntityId, EventEmitter,
    IntoElement, Model, ParentElement, Pixels, SharedString, Styled, Task, View, ViewContext,
    VisualContext, WeakView, WindowContext,
};
use language::{
    proto::serialize_anchor as serialize_text_anchor, Bias, Buffer, CharKind, DiskState, Point,
    SelectionGoal,
};
use lsp::DiagnosticSeverity;
use multi_buffer::AnchorRangeExt;
use project::{
    lsp_store::FormatTrigger, project_settings::ProjectSettings, search::SearchQuery, Project,
    ProjectItem as _, ProjectPath,
};
use rpc::proto::{self, update_view, PeerId};
use settings::Settings;
use workspace::item::{Dedup, ItemSettings, SerializableItem, TabContentParams};

use project::lsp_store::FormatTarget;
use std::{
    any::TypeId,
    borrow::Cow,
    cmp::{self, Ordering},
    iter,
    ops::Range,
    path::Path,
    sync::Arc,
};
use text::{BufferId, Selection};
use theme::{Theme, ThemeSettings};
use ui::{h_flex, prelude::*, IconDecorationKind, Label};
use util::{paths::PathExt, ResultExt, TryFutureExt};
use workspace::item::{BreadcrumbText, FollowEvent};
use workspace::{
    item::{FollowableItem, Item, ItemEvent, ProjectItem},
    searchable::{Direction, SearchEvent, SearchableItem, SearchableItemHandle},
    ItemId, ItemNavHistory, ToolbarItemLocation, ViewId, Workspace, WorkspaceId,
};

pub const MAX_TAB_TITLE_LEN: usize = 24;

impl FollowableItem for Editor {
    fn remote_id(&self) -> Option<ViewId> {
        self.remote_id
    }

    fn from_state_proto(
        workspace: View<Workspace>,
        remote_id: ViewId,
        state: &mut Option<proto::view::Variant>,
        cx: &mut WindowContext,
    ) -> Option<Task<Result<View<Self>>>> {
        let project = workspace.read(cx).project().to_owned();
        let Some(proto::view::Variant::Editor(_)) = state else {
            return None;
        };
        let Some(proto::view::Variant::Editor(state)) = state.take() else {
            unreachable!()
        };

        let buffer_ids = state
            .excerpts
            .iter()
            .map(|excerpt| excerpt.buffer_id)
            .collect::<HashSet<_>>();
        let buffers = project.update(cx, |project, cx| {
            buffer_ids
                .iter()
                .map(|id| BufferId::new(*id).map(|id| project.open_buffer_by_id(id, cx)))
                .collect::<Result<Vec<_>>>()
        });

        Some(cx.spawn(|mut cx| async move {
            let mut buffers = futures::future::try_join_all(buffers?)
                .await
                .debug_assert_ok("leaders don't share views for unshared buffers")?;

            let editor = cx.update(|cx| {
                let multibuffer = cx.new_model(|cx| {
                    let mut multibuffer;
                    if state.singleton && buffers.len() == 1 {
                        multibuffer = MultiBuffer::singleton(buffers.pop().unwrap(), cx)
                    } else {
                        multibuffer = MultiBuffer::new(project.read(cx).capability());
                        let mut excerpts = state.excerpts.into_iter().peekable();
                        while let Some(excerpt) = excerpts.peek() {
                            let Ok(buffer_id) = BufferId::new(excerpt.buffer_id) else {
                                continue;
                            };
                            let buffer_excerpts = iter::from_fn(|| {
                                let excerpt = excerpts.peek()?;
                                (excerpt.buffer_id == u64::from(buffer_id))
                                    .then(|| excerpts.next().unwrap())
                            });
                            let buffer =
                                buffers.iter().find(|b| b.read(cx).remote_id() == buffer_id);
                            if let Some(buffer) = buffer {
                                multibuffer.push_excerpts(
                                    buffer.clone(),
                                    buffer_excerpts.filter_map(deserialize_excerpt_range),
                                    cx,
                                );
                            }
                        }
                    };

                    if let Some(title) = &state.title {
                        multibuffer = multibuffer.with_title(title.clone())
                    }

                    multibuffer
                });

                cx.new_view(|cx| {
                    let mut editor =
                        Editor::for_multibuffer(multibuffer, Some(project.clone()), true, cx);
                    editor.remote_id = Some(remote_id);
                    editor
                })
            })?;

            update_editor_from_message(
                editor.downgrade(),
                project,
                proto::update_view::Editor {
                    selections: state.selections,
                    pending_selection: state.pending_selection,
                    scroll_top_anchor: state.scroll_top_anchor,
                    scroll_x: state.scroll_x,
                    scroll_y: state.scroll_y,
                    ..Default::default()
                },
                &mut cx,
            )
            .await?;

            Ok(editor)
        }))
    }

    fn set_leader_peer_id(&mut self, leader_peer_id: Option<PeerId>, cx: &mut ViewContext<Self>) {
        self.leader_peer_id = leader_peer_id;
        if self.leader_peer_id.is_some() {
            self.buffer.update(cx, |buffer, cx| {
                buffer.remove_active_selections(cx);
            });
        } else if self.focus_handle.is_focused(cx) {
            self.buffer.update(cx, |buffer, cx| {
                buffer.set_active_selections(
                    &self.selections.disjoint_anchors(),
                    self.selections.line_mode,
                    self.cursor_shape,
                    cx,
                );
            });
        }
        cx.notify();
    }

    fn to_state_proto(&self, cx: &WindowContext) -> Option<proto::view::Variant> {
        let buffer = self.buffer.read(cx);
        if buffer
            .as_singleton()
            .and_then(|buffer| buffer.read(cx).file())
            .map_or(false, |file| file.is_private())
        {
            return None;
        }

        let scroll_anchor = self.scroll_manager.anchor();
        let excerpts = buffer
            .read(cx)
            .excerpts()
            .map(|(id, buffer, range)| proto::Excerpt {
                id: id.to_proto(),
                buffer_id: buffer.remote_id().into(),
                context_start: Some(serialize_text_anchor(&range.context.start)),
                context_end: Some(serialize_text_anchor(&range.context.end)),
                primary_start: range
                    .primary
                    .as_ref()
                    .map(|range| serialize_text_anchor(&range.start)),
                primary_end: range
                    .primary
                    .as_ref()
                    .map(|range| serialize_text_anchor(&range.end)),
            })
            .collect();

        Some(proto::view::Variant::Editor(proto::view::Editor {
            singleton: buffer.is_singleton(),
            title: (!buffer.is_singleton()).then(|| buffer.title(cx).into()),
            excerpts,
            scroll_top_anchor: Some(serialize_anchor(&scroll_anchor.anchor)),
            scroll_x: scroll_anchor.offset.x,
            scroll_y: scroll_anchor.offset.y,
            selections: self
                .selections
                .disjoint_anchors()
                .iter()
                .map(serialize_selection)
                .collect(),
            pending_selection: self
                .selections
                .pending_anchor()
                .as_ref()
                .map(serialize_selection),
        }))
    }

    fn to_follow_event(event: &EditorEvent) -> Option<workspace::item::FollowEvent> {
        match event {
            EditorEvent::Edited { .. } => Some(FollowEvent::Unfollow),
            EditorEvent::SelectionsChanged { local }
            | EditorEvent::ScrollPositionChanged { local, .. } => {
                if *local {
                    Some(FollowEvent::Unfollow)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn add_event_to_update_proto(
        &self,
        event: &EditorEvent,
        update: &mut Option<proto::update_view::Variant>,
        cx: &WindowContext,
    ) -> bool {
        let update =
            update.get_or_insert_with(|| proto::update_view::Variant::Editor(Default::default()));

        match update {
            proto::update_view::Variant::Editor(update) => match event {
                EditorEvent::ExcerptsAdded {
                    buffer,
                    predecessor,
                    excerpts,
                } => {
                    let buffer_id = buffer.read(cx).remote_id();
                    let mut excerpts = excerpts.iter();
                    if let Some((id, range)) = excerpts.next() {
                        update.inserted_excerpts.push(proto::ExcerptInsertion {
                            previous_excerpt_id: Some(predecessor.to_proto()),
                            excerpt: serialize_excerpt(buffer_id, id, range),
                        });
                        update.inserted_excerpts.extend(excerpts.map(|(id, range)| {
                            proto::ExcerptInsertion {
                                previous_excerpt_id: None,
                                excerpt: serialize_excerpt(buffer_id, id, range),
                            }
                        }))
                    }
                    true
                }
                EditorEvent::ExcerptsRemoved { ids } => {
                    update
                        .deleted_excerpts
                        .extend(ids.iter().map(ExcerptId::to_proto));
                    true
                }
                EditorEvent::ScrollPositionChanged { autoscroll, .. } if !autoscroll => {
                    let scroll_anchor = self.scroll_manager.anchor();
                    update.scroll_top_anchor = Some(serialize_anchor(&scroll_anchor.anchor));
                    update.scroll_x = scroll_anchor.offset.x;
                    update.scroll_y = scroll_anchor.offset.y;
                    true
                }
                EditorEvent::SelectionsChanged { .. } => {
                    update.selections = self
                        .selections
                        .disjoint_anchors()
                        .iter()
                        .map(serialize_selection)
                        .collect();
                    update.pending_selection = self
                        .selections
                        .pending_anchor()
                        .as_ref()
                        .map(serialize_selection);
                    true
                }
                _ => false,
            },
        }
    }

    fn apply_update_proto(
        &mut self,
        project: &Model<Project>,
        message: update_view::Variant,
        cx: &mut ViewContext<Self>,
    ) -> Task<Result<()>> {
        let update_view::Variant::Editor(message) = message;
        let project = project.clone();
        cx.spawn(|this, mut cx| async move {
            update_editor_from_message(this, project, message, &mut cx).await
        })
    }

    fn is_project_item(&self, _cx: &WindowContext) -> bool {
        true
    }

    fn dedup(&self, existing: &Self, cx: &WindowContext) -> Option<Dedup> {
        let self_singleton = self.buffer.read(cx).as_singleton()?;
        let other_singleton = existing.buffer.read(cx).as_singleton()?;
        if self_singleton == other_singleton {
            Some(Dedup::KeepExisting)
        } else {
            None
        }
    }
}

async fn update_editor_from_message(
    this: WeakView<Editor>,
    project: Model<Project>,
    message: proto::update_view::Editor,
    cx: &mut AsyncWindowContext,
) -> Result<()> {
    // Open all of the buffers of which excerpts were added to the editor.
    let inserted_excerpt_buffer_ids = message
        .inserted_excerpts
        .iter()
        .filter_map(|insertion| Some(insertion.excerpt.as_ref()?.buffer_id))
        .collect::<HashSet<_>>();
    let inserted_excerpt_buffers = project.update(cx, |project, cx| {
        inserted_excerpt_buffer_ids
            .into_iter()
            .map(|id| BufferId::new(id).map(|id| project.open_buffer_by_id(id, cx)))
            .collect::<Result<Vec<_>>>()
    })??;
    let _inserted_excerpt_buffers = try_join_all(inserted_excerpt_buffers).await?;

    // Update the editor's excerpts.
    this.update(cx, |editor, cx| {
        editor.buffer.update(cx, |multibuffer, cx| {
            let mut removed_excerpt_ids = message
                .deleted_excerpts
                .into_iter()
                .map(ExcerptId::from_proto)
                .collect::<Vec<_>>();
            removed_excerpt_ids.sort_by({
                let multibuffer = multibuffer.read(cx);
                move |a, b| a.cmp(b, &multibuffer)
            });

            let mut insertions = message.inserted_excerpts.into_iter().peekable();
            while let Some(insertion) = insertions.next() {
                let Some(excerpt) = insertion.excerpt else {
                    continue;
                };
                let Some(previous_excerpt_id) = insertion.previous_excerpt_id else {
                    continue;
                };
                let buffer_id = BufferId::new(excerpt.buffer_id)?;
                let Some(buffer) = project.read(cx).buffer_for_id(buffer_id, cx) else {
                    continue;
                };

                let adjacent_excerpts = iter::from_fn(|| {
                    let insertion = insertions.peek()?;
                    if insertion.previous_excerpt_id.is_none()
                        && insertion.excerpt.as_ref()?.buffer_id == u64::from(buffer_id)
                    {
                        insertions.next()?.excerpt
                    } else {
                        None
                    }
                });

                multibuffer.insert_excerpts_with_ids_after(
                    ExcerptId::from_proto(previous_excerpt_id),
                    buffer,
                    [excerpt]
                        .into_iter()
                        .chain(adjacent_excerpts)
                        .filter_map(|excerpt| {
                            Some((
                                ExcerptId::from_proto(excerpt.id),
                                deserialize_excerpt_range(excerpt)?,
                            ))
                        }),
                    cx,
                );
            }

            multibuffer.remove_excerpts(removed_excerpt_ids, cx);
            Result::<(), anyhow::Error>::Ok(())
        })
    })??;

    // Deserialize the editor state.
    let (selections, pending_selection, scroll_top_anchor) = this.update(cx, |editor, cx| {
        let buffer = editor.buffer.read(cx).read(cx);
        let selections = message
            .selections
            .into_iter()
            .filter_map(|selection| deserialize_selection(&buffer, selection))
            .collect::<Vec<_>>();
        let pending_selection = message
            .pending_selection
            .and_then(|selection| deserialize_selection(&buffer, selection));
        let scroll_top_anchor = message
            .scroll_top_anchor
            .and_then(|anchor| deserialize_anchor(&buffer, anchor));
        anyhow::Ok((selections, pending_selection, scroll_top_anchor))
    })??;

    // Wait until the buffer has received all of the operations referenced by
    // the editor's new state.
    this.update(cx, |editor, cx| {
        editor.buffer.update(cx, |buffer, cx| {
            buffer.wait_for_anchors(
                selections
                    .iter()
                    .chain(pending_selection.as_ref())
                    .flat_map(|selection| [selection.start, selection.end])
                    .chain(scroll_top_anchor),
                cx,
            )
        })
    })?
    .await?;

    // Update the editor's state.
    this.update(cx, |editor, cx| {
        if !selections.is_empty() || pending_selection.is_some() {
            editor.set_selections_from_remote(selections, pending_selection, cx);
            editor.request_autoscroll_remotely(Autoscroll::newest(), cx);
        } else if let Some(scroll_top_anchor) = scroll_top_anchor {
            editor.set_scroll_anchor_remote(
                ScrollAnchor {
                    anchor: scroll_top_anchor,
                    offset: point(message.scroll_x, message.scroll_y),
                },
                cx,
            );
        }
    })?;
    Ok(())
}

fn serialize_excerpt(
    buffer_id: BufferId,
    id: &ExcerptId,
    range: &ExcerptRange<language::Anchor>,
) -> Option<proto::Excerpt> {
    Some(proto::Excerpt {
        id: id.to_proto(),
        buffer_id: buffer_id.into(),
        context_start: Some(serialize_text_anchor(&range.context.start)),
        context_end: Some(serialize_text_anchor(&range.context.end)),
        primary_start: range
            .primary
            .as_ref()
            .map(|r| serialize_text_anchor(&r.start)),
        primary_end: range
            .primary
            .as_ref()
            .map(|r| serialize_text_anchor(&r.end)),
    })
}

fn serialize_selection(selection: &Selection<Anchor>) -> proto::Selection {
    proto::Selection {
        id: selection.id as u64,
        start: Some(serialize_anchor(&selection.start)),
        end: Some(serialize_anchor(&selection.end)),
        reversed: selection.reversed,
    }
}

fn serialize_anchor(anchor: &Anchor) -> proto::EditorAnchor {
    proto::EditorAnchor {
        excerpt_id: anchor.excerpt_id.to_proto(),
        anchor: Some(serialize_text_anchor(&anchor.text_anchor)),
    }
}

fn deserialize_excerpt_range(excerpt: proto::Excerpt) -> Option<ExcerptRange<language::Anchor>> {
    let context = {
        let start = language::proto::deserialize_anchor(excerpt.context_start?)?;
        let end = language::proto::deserialize_anchor(excerpt.context_end?)?;
        start..end
    };
    let primary = excerpt
        .primary_start
        .zip(excerpt.primary_end)
        .and_then(|(start, end)| {
            let start = language::proto::deserialize_anchor(start)?;
            let end = language::proto::deserialize_anchor(end)?;
            Some(start..end)
        });
    Some(ExcerptRange { context, primary })
}

fn deserialize_selection(
    buffer: &MultiBufferSnapshot,
    selection: proto::Selection,
) -> Option<Selection<Anchor>> {
    Some(Selection {
        id: selection.id as usize,
        start: deserialize_anchor(buffer, selection.start?)?,
        end: deserialize_anchor(buffer, selection.end?)?,
        reversed: selection.reversed,
        goal: SelectionGoal::None,
    })
}

fn deserialize_anchor(buffer: &MultiBufferSnapshot, anchor: proto::EditorAnchor) -> Option<Anchor> {
    let excerpt_id = ExcerptId::from_proto(anchor.excerpt_id);
    Some(Anchor {
        excerpt_id,
        text_anchor: language::proto::deserialize_anchor(anchor.anchor?)?,
        buffer_id: buffer.buffer_id_for_excerpt(excerpt_id),
    })
}

impl Item for Editor {
    type Event = EditorEvent;

    fn navigate(&mut self, data: Box<dyn std::any::Any>, cx: &mut ViewContext<Self>) -> bool {
        if let Ok(data) = data.downcast::<NavigationData>() {
            let newest_selection = self.selections.newest::<Point>(cx);
            let buffer = self.buffer.read(cx).read(cx);
            let offset = if buffer.can_resolve(&data.cursor_anchor) {
                data.cursor_anchor.to_point(&buffer)
            } else {
                buffer.clip_point(data.cursor_position, Bias::Left)
            };

            let mut scroll_anchor = data.scroll_anchor;
            if !buffer.can_resolve(&scroll_anchor.anchor) {
                scroll_anchor.anchor = buffer.anchor_before(
                    buffer.clip_point(Point::new(data.scroll_top_row, 0), Bias::Left),
                );
            }

            drop(buffer);

            if newest_selection.head() == offset {
                false
            } else {
                let nav_history = self.nav_history.take();
                self.set_scroll_anchor(scroll_anchor, cx);
                self.change_selections(Some(Autoscroll::fit()), cx, |s| {
                    s.select_ranges([offset..offset])
                });
                self.nav_history = nav_history;
                true
            }
        } else {
            false
        }
    }

    fn tab_tooltip_text(&self, cx: &AppContext) -> Option<SharedString> {
        let file_path = self
            .buffer()
            .read(cx)
            .as_singleton()?
            .read(cx)
            .file()
            .and_then(|f| f.as_local())?
            .abs_path(cx);

        let file_path = file_path.compact().to_string_lossy().to_string();

        Some(file_path.into())
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        None
    }

    fn tab_description(&self, detail: usize, cx: &AppContext) -> Option<SharedString> {
        let path = path_for_buffer(&self.buffer, detail, true, cx)?;
        Some(path.to_string_lossy().to_string().into())
    }

    fn tab_icon(&self, cx: &WindowContext) -> Option<Icon> {
        ItemSettings::get_global(cx)
            .file_icons
            .then(|| {
                self.buffer
                    .read(cx)
                    .as_singleton()
                    .and_then(|buffer| buffer.read(cx).project_path(cx))
                    .and_then(|path| FileIcons::get_icon(path.path.as_ref(), cx))
            })
            .flatten()
            .map(Icon::from_path)
    }

    fn tab_content(&self, params: TabContentParams, cx: &WindowContext) -> AnyElement {
        let label_color = if ItemSettings::get_global(cx).git_status {
            self.buffer()
                .read(cx)
                .as_singleton()
                .and_then(|buffer| buffer.read(cx).project_path(cx))
                .and_then(|path| self.project.as_ref()?.read(cx).entry_for_path(&path, cx))
                .map(|entry| {
                    entry_git_aware_label_color(entry.git_status, entry.is_ignored, params.selected)
                })
                .unwrap_or_else(|| entry_label_color(params.selected))
        } else {
            entry_label_color(params.selected)
        };

        let description = params.detail.and_then(|detail| {
            let path = path_for_buffer(&self.buffer, detail, false, cx)?;
            let description = path.to_string_lossy();
            let description = description.trim();

            if description.is_empty() {
                return None;
            }

            Some(util::truncate_and_trailoff(description, MAX_TAB_TITLE_LEN))
        });

        // Whether the file was saved in the past but is now deleted.
        let was_deleted: bool = self
            .buffer()
            .read(cx)
            .as_singleton()
            .and_then(|buffer| buffer.read(cx).file())
            .map_or(false, |file| file.disk_state() == DiskState::Deleted);

        h_flex()
            .gap_2()
            .child(
                Label::new(self.title(cx).to_string())
                    .color(label_color)
                    .italic(params.preview)
                    .strikethrough(was_deleted),
            )
            .when_some(description, |this, description| {
                this.child(
                    Label::new(description)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                )
            })
            .into_any_element()
    }

    fn for_each_project_item(
        &self,
        cx: &AppContext,
        f: &mut dyn FnMut(EntityId, &dyn project::ProjectItem),
    ) {
        self.buffer
            .read(cx)
            .for_each_buffer(|buffer| f(buffer.entity_id(), buffer.read(cx)));
    }

    fn is_singleton(&self, cx: &AppContext) -> bool {
        self.buffer.read(cx).is_singleton()
    }

    fn clone_on_split(
        &self,
        _workspace_id: Option<WorkspaceId>,
        cx: &mut ViewContext<Self>,
    ) -> Option<View<Editor>>
    where
        Self: Sieditsync,
    {
        Some(cx.new_view(|cx| self.clone(cx)))
    }

    fn set_nav_history(&mut self, history: ItemNavHistory, _: &mut ViewContext<Self>) {
        self.nav_history = Some(history);
    }

    fn discarded(&self, _project: Model<Project>, cx: &mut ViewContext<Self>) {
        for buffer in self.buffer().clone().read(cx).all_buffers() {
            buffer.update(cx, |buffer, cx| buffer.discarded(cx))
        }
    }

    fn deactivated(&mut self, cx: &mut ViewContext<Self>) {
        let selection = self.selections.newest_anchor();
        self.push_to_nav_history(selection.head(), None, cx);
    }

    fn workspace_deactivated(&mut self, cx: &mut ViewContext<Self>) {
        self.hide_hovered_link(cx);
    }

    fn is_dirty(&self, cx: &AppContext) -> bool {
        self.buffer().read(cx).read(cx).is_dirty()
    }

    fn has_deleted_file(&self, cx: &AppContext) -> bool {
        self.buffer().read(cx).read(cx).has_deleted_file()
    }

    fn has_conflict(&self, cx: &AppContext) -> bool {
        self.buffer().read(cx).read(cx).has_conflict()
    }

    fn can_save(&self, cx: &AppContext) -> bool {
        let buffer = &self.buffer().read(cx);
        if let Some(buffer) = buffer.as_singleton() {
            buffer.read(cx).project_path(cx).is_some()
        } else {
            true
        }
    }

    fn save(
        &mut self,
        format: bool,
        project: Model<Project>,
        cx: &mut ViewContext<Self>,
    ) -> Task<Result<()>> {
        self.report_editor_event("Editor Saved", None, cx);
        let buffers = self.buffer().clone().read(cx).all_buffers();
        let buffers = buffers
            .into_iter()
            .map(|handle| handle.read(cx).base_buffer().unwrap_or(handle.clone()))
            .collect::<HashSet<_>>();
        cx.spawn(|this, mut cx| async move {
            if format {
                this.update(&mut cx, |editor, cx| {
                    editor.perform_format(
                        project.clone(),
                        FormatTrigger::Save,
                        FormatTarget::Buffer,
                        cx,
                    )
                })?
                .await?;
            }

            if buffers.len() == 1 {
                // Apply full save routine for singleton buffers, to allow to `touch` the file via the editor.
                project
                    .update(&mut cx, |project, cx| project.save_buffers(buffers, cx))?
                    .await?;
            } else {
                // For multi-buffers, only format and save the buffers with changes.
                // For clean buffers, we simulate saving by calling `Buffer::did_save`,
                // so that language servers or other downstream listeners of save events get notified.
                let (dirty_buffers, clean_buffers) = buffers.into_iter().partition(|buffer| {
                    buffer
                        .update(&mut cx, |buffer, _| {
                            buffer.is_dirty() || buffer.has_conflict()
                        })
                        .unwrap_or(false)
                });

                project
                    .update(&mut cx, |project, cx| {
                        project.save_buffers(dirty_buffers, cx)
                    })?
                    .await?;
                for buffer in clean_buffers {
                    buffer
                        .update(&mut cx, |buffer, cx| {
                            let version = buffer.saved_version().clone();
                            let mtime = buffer.saved_mtime();
                            buffer.did_save(version, mtime, cx);
                        })
                        .ok();
                }
            }

            Ok(())
        })
    }

    fn save_as(
        &mut self,
        project: Model<Project>,
        path: ProjectPath,
        cx: &mut ViewContext<Self>,
    ) -> Task<Result<()>> {
        let buffer = self
            .buffer()
            .read(cx)
            .as_singleton()
            .expect("cannot call save_as on an excerpt list");

        let file_extension = path
            .path
            .extension()
            .map(|a| a.to_string_lossy().to_string());
        self.report_editor_event("Editor Saved", file_extension, cx);

        project.update(cx, |project, cx| project.save_buffer_as(buffer, path, cx))
    }

    fn reload(&mut self, project: Model<Project>, cx: &mut ViewContext<Self>) -> Task<Result<()>> {
        let buffer = self.buffer().clone();
        let buffers = self.buffer.read(cx).all_buffers();
        let reload_buffers =
            project.update(cx, |project, cx| project.reload_buffers(buffers, true, cx));
        cx.spawn(|this, mut cx| async move {
            let transaction = reload_buffers.log_err().await;
            this.update(&mut cx, |editor, cx| {
                editor.request_autoscroll(Autoscroll::fit(), cx)
            })?;
            buffer
                .update(&mut cx, |buffer, cx| {
                    if let Some(transaction) = transaction {
                        if !buffer.is_singleton() {
                            buffer.push_transaction(&transaction.0, cx);
                        }
                    }
                })
                .ok();
            Ok(())
        })
    }

    fn as_searchable(&self, handle: &View<Self>) -> Option<Box<dyn SearchableItemHandle>> {
        Some(Box::new(handle.clone()))
    }

    fn pixel_position_of_cursor(&self, _: &AppContext) -> Option<gpui::Point<Pixels>> {
        self.pixel_position_of_newest_cursor
    }

    fn breadcrumb_location(&self, _: &AppContext) -> ToolbarItemLocation {
        if self.show_breadcrumbs {
            ToolbarItemLocation::PrimaryLeft
        } else {
            ToolbarItemLocation::Hidden
        }
    }

    fn breadcrumbs(&self, variant: &Theme, cx: &AppContext) -> Option<Vec<BreadcrumbText>> {
        let cursor = self.selections.newest_anchor().head();
        let multibuffer = &self.buffer().read(cx);
        let (buffer_id, symbols) =
            multibuffer.symbols_containing(cursor, Some(variant.syntax()), cx)?;
        let buffer = multibuffer.buffer(buffer_id)?;

        let buffer = buffer.read(cx);
        let text = self.breadcrumb_header.clone().unwrap_or_else(|| {
            buffer
                .snapshot()
                .resolve_file_path(
                    cx,
                    self.project
                        .as_ref()
                        .map(|project| project.read(cx).visible_worktrees(cx).count() > 1)
                        .unwrap_or_default(),
                )
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    if multibuffer.is_singleton() {
                        multibuffer.title(cx).to_string()
                    } else {
                        "untitled".to_string()
                    }
                })
        });

        let settings = ThemeSettings::get_global(cx);

        let mut breadcrumbs = vec![BreadcrumbText {
            text,
            highlights: None,
            font: Some(settings.buffer_font.clone()),
        }];

        breadcrumbs.extend(symbols.into_iter().map(|symbol| BreadcrumbText {
            text: symbol.text,
            highlights: Some(symbol.highlight_ranges),
            font: Some(settings.buffer_font.clone()),
        }));
        Some(breadcrumbs)
    }

    fn added_to_workspace(&mut self, workspace: &mut Workspace, _: &mut ViewContext<Self>) {
        self.workspace = Some((workspace.weak_handle(), workspace.database_id()));
    }

    fn to_item_events(event: &EditorEvent, mut f: impl FnMut(ItemEvent)) {
        match event {
            EditorEvent::Closed => f(ItemEvent::CloseItem),

            EditorEvent::Saved | EditorEvent::TitleChanged => {
                f(ItemEvent::UpdateTab);
                f(ItemEvent::UpdateBreadcrumbs);
            }

            EditorEvent::Reparsed(_) => {
                f(ItemEvent::UpdateBreadcrumbs);
            }

            EditorEvent::SelectionsChanged { local } if *local => {
                f(ItemEvent::UpdateBreadcrumbs);
            }

            EditorEvent::DirtyChanged => {
                f(ItemEvent::UpdateTab);
            }

            EditorEvent::BufferEdited => {
                f(ItemEvent::Edit);
                f(ItemEvent::UpdateBreadcrumbs);
            }

            EditorEvent::ExcerptsAdded { .. } | EditorEvent::ExcerptsRemoved { .. } => {
                f(ItemEvent::Edit);
            }

            _ => {}
        }
    }

    fn preserve_preview(&self, cx: &AppContext) -> bool {
        self.buffer.read(cx).preserve_preview(cx)
    }
}

impl SerializableItem for Editor {
    fn serialieditsync_item_kind() -> &'static str {
        "Editor"
    }

    fn cleanup(
        workspace_id: WorkspaceId,
        alive_items: Vec<ItemId>,
        cx: &mut WindowContext,
    ) -> Task<Result<()>> {
        cx.spawn(|_| DB.delete_unloaded_items(workspace_id, alive_items))
    }

    fn deserialize(
        project: Model<Project>,
        workspace: WeakView<Workspace>,
        workspace_id: workspace::WorkspaceId,
        item_id: ItemId,
        cx: &mut WindowContext,
    ) -> Task<Result<View<Self>>> {
        let serialieditsync_editor = match DB
            .get_serialieditsync_editor(item_id, workspace_id)
            .context("Failed to query editor state")
        {
            Ok(Some(serialieditsync_editor)) => {
                if ProjectSettings::get_global(cx)
                    .session
                    .restore_unsaved_buffers
                {
                    serialieditsync_editor
                } else {
                    SerialieditsyncEditor {
                        abs_path: serialieditsync_editor.abs_path,
                        contents: None,
                        language: None,
                        mtime: None,
                    }
                }
            }
            Ok(None) => {
                return Task::ready(Err(anyhow!("No path or contents found for buffer")));
            }
            Err(error) => {
                return Task::ready(Err(error));
            }
        };

        match serialieditsync_editor {
            SerialieditsyncEditor {
                abs_path: None,
                contents: Some(contents),
                language,
                ..
            } => cx.spawn(|mut cx| {
                let project = project.clone();
                async move {
                    let language = if let Some(language_name) = language {
                        let language_registry =
                            project.update(&mut cx, |project, _| project.languages().clone())?;

                        // We don't fail here, because we'd rather not set the language if the name changed
                        // than fail to restore the buffer.
                        language_registry
                            .language_for_name(&language_name)
                            .await
                            .ok()
                    } else {
                        None
                    };

                    // First create the empty buffer
                    let buffer = project
                        .update(&mut cx, |project, cx| project.create_buffer(cx))?
                        .await?;

                    // Then set the text so that the dirty bit is set correctly
                    buffer.update(&mut cx, |buffer, cx| {
                        if let Some(language) = language {
                            buffer.set_language(Some(language), cx);
                        }
                        buffer.set_text(contents, cx);
                    })?;

                    cx.update(|cx| {
                        cx.new_view(|cx| {
                            let mut editor = Editor::for_buffer(buffer, Some(project), cx);

                            editor.read_scroll_position_from_db(item_id, workspace_id, cx);
                            editor
                        })
                    })
                }
            }),
            SerialieditsyncEditor {
                abs_path: Some(abs_path),
                contents,
                mtime,
                ..
            } => {
                let project_item = project.update(cx, |project, cx| {
                    let (worktree, path) = project.find_worktree(&abs_path, cx)?;
                    let project_path = ProjectPath {
                        worktree_id: worktree.read(cx).id(),
                        path: path.into(),
                    };
                    Some(project.open_path(project_path, cx))
                });

                match project_item {
                    Some(project_item) => {
                        cx.spawn(|mut cx| async move {
                            let (_, project_item) = project_item.await?;
                            let buffer = project_item.downcast::<Buffer>().map_err(|_| {
                                anyhow!("Project item at stored path was not a buffer")
                            })?;

                            // This is a bit wasteful: we're loading the whole buffer from
                            // disk and then overwrite the content.
                            // But for now, it keeps the implementation of the content serialization
                            // simple, because we don't have to persist all of the metadata that we get
                            // by loading the file (git diff base, ...).
                            if let Some(buffer_text) = contents {
                                buffer.update(&mut cx, |buffer, cx| {
                                    // If we did restore an mtime, we want to store it on the buffer
                                    // so that the next edit will mark the buffer as dirty/conflicted.
                                    if mtime.is_some() {
                                        buffer.did_reload(
                                            buffer.version(),
                                            buffer.line_ending(),
                                            mtime,
                                            cx,
                                        );
                                    }
                                    buffer.set_text(buffer_text, cx);
                                })?;
                            }

                            cx.update(|cx| {
                                cx.new_view(|cx| {
                                    let mut editor = Editor::for_buffer(buffer, Some(project), cx);

                                    editor.read_scroll_position_from_db(item_id, workspace_id, cx);
                                    editor
                                })
                            })
                        })
                    }
                    None => {
                        let open_by_abs_path = workspace.update(cx, |workspace, cx| {
                            workspace.open_abs_path(abs_path.clone(), false, cx)
                        });
                        cx.spawn(|mut cx| async move {
                            let editor = open_by_abs_path?.await?.downcast::<Editor>().with_context(|| format!("Failed to downcast to Editor after opening abs path {abs_path:?}"))?;
                            editor.update(&mut cx, |editor, cx| {
                                editor.read_scroll_position_from_db(item_id, workspace_id, cx);
                            })?;
                            Ok(editor)
                        })
                    }
                }
            }
            SerialieditsyncEditor {
                abs_path: None,
                contents: None,
                ..
            } => Task::ready(Err(anyhow!("No path or contents found for buffer"))),
        }
    }

    fn serialize(
        &mut self,
        workspace: &mut Workspace,
        item_id: ItemId,
        closing: bool,
        cx: &mut ViewContext<Self>,
    ) -> Option<Task<Result<()>>> {
        let mut serialize_dirty_buffers = self.serialize_dirty_buffers;

        let project = self.project.clone()?;
        if project.read(cx).visible_worktrees(cx).next().is_none() {
            // If we don't have a worktree, we don't serialize, because
            // projects without worktrees aren't deserialieditsync.
            serialize_dirty_buffers = false;
        }

        if closing && !serialize_dirty_buffers {
            return None;
        }

        let workspace_id = workspace.database_id()?;

        let buffer = self.buffer().read(cx).as_singleton()?;

        let abs_path = buffer.read(cx).file().and_then(|file| {
            let worktree_id = file.worktree_id(cx);
            project
                .read(cx)
                .worktree_for_id(worktree_id, cx)
                .and_then(|worktree| worktree.read(cx).absolutize(&file.path()).ok())
                .or_else(|| {
                    let full_path = file.full_path(cx);
                    let project_path = project.read(cx).find_project_path(&full_path, cx)?;
                    project.read(cx).absolute_path(&project_path, cx)
                })
        });

        let is_dirty = buffer.read(cx).is_dirty();
        let mtime = buffer.read(cx).saved_mtime();

        let snapshot = buffer.read(cx).snapshot();

        Some(cx.spawn(|_this, cx| async move {
            cx.background_executor()
                .spawn(async move {
                    let (contents, language) = if serialize_dirty_buffers && is_dirty {
                        let contents = snapshot.text();
                        let language = snapshot.language().map(|lang| lang.name().to_string());
                        (Some(contents), language)
                    } else {
                        (None, None)
                    };

                    let editor = SerialieditsyncEditor {
                        abs_path,
                        contents,
                        language,
                        mtime,
                    };

                    DB.save_serialieditsync_editor(item_id, workspace_id, editor)
                        .await
                        .context("failed to save serialieditsync editor")
                })
                .await
                .context("failed to save contents of buffer")?;

            Ok(())
        }))
    }

    fn should_serialize(&self, event: &Self::Event) -> bool {
        matches!(
            event,
            EditorEvent::Saved | EditorEvent::DirtyChanged | EditorEvent::BufferEdited
        )
    }
}

impl ProjectItem for Editor {
    type Item = Buffer;

    fn for_project_item(
        project: Model<Project>,
        buffer: Model<Buffer>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        Self::for_buffer(buffer, Some(project), cx)
    }
}

impl EventEmitter<SearchEvent> for Editor {}

pub(crate) enum BufferSearchHighlights {}
impl SearchableItem for Editor {
    type Match = Range<Anchor>;

    fn get_matches(&self, _: &mut WindowContext) -> Vec<Range<Anchor>> {
        self.background_highlights
            .get(&TypeId::of::<BufferSearchHighlights>())
            .map_or(Vec::new(), |(_color, ranges)| {
                ranges.iter().cloned().collect()
            })
    }

    fn clear_matches(&mut self, cx: &mut ViewContext<Self>) {
        if self
            .clear_background_highlights::<BufferSearchHighlights>(cx)
            .is_some()
        {
            cx.emit(SearchEvent::MatchesInvalidated);
        }
    }

    fn update_matches(&mut self, matches: &[Range<Anchor>], cx: &mut ViewContext<Self>) {
        let existing_range = self
            .background_highlights
            .get(&TypeId::of::<BufferSearchHighlights>())
            .map(|(_, range)| range.as_ref());
        let updated = existing_range != Some(matches);
        self.highlight_background::<BufferSearchHighlights>(
            matches,
            |theme| theme.search_match_background,
            cx,
        );
        if updated {
            cx.emit(SearchEvent::MatchesInvalidated);
        }
    }

    fn has_filtered_search_ranges(&mut self) -> bool {
        self.has_background_highlights::<SearchWithinRange>()
    }

    fn toggle_filtered_search_ranges(&mut self, enabled: bool, cx: &mut ViewContext<Self>) {
        if self.has_filtered_search_ranges() {
            self.previous_search_ranges = self
                .clear_background_highlights::<SearchWithinRange>(cx)
                .map(|(_, ranges)| ranges)
        }

        if !enabled {
            return;
        }

        let ranges = self.selections.disjoint_anchor_ranges();
        if ranges.iter().any(|range| range.start != range.end) {
            self.set_search_within_ranges(&ranges, cx);
        } else if let Some(previous_search_ranges) = self.previous_search_ranges.take() {
            self.set_search_within_ranges(&previous_search_ranges, cx)
        }
    }

    fn query_suggestion(&mut self, cx: &mut ViewContext<Self>) -> String {
        let setting = EditorSettings::get_global(cx).seed_search_query_from_cursor;
        let snapshot = &self.snapshot(cx).buffer_snapshot;
        let selection = self.selections.newest::<usize>(cx);

        match setting {
            SeedQuerySetting::Never => String::new(),
            SeedQuerySetting::Selection | SeedQuerySetting::Always if !selection.is_empty() => {
                let text: String = snapshot
                    .text_for_range(selection.start..selection.end)
                    .collect();
                if text.contains('\n') {
                    String::new()
                } else {
                    text
                }
            }
            SeedQuerySetting::Selection => String::new(),
            SeedQuerySetting::Always => {
                let (range, kind) = snapshot.surrounding_word(selection.start, true);
                if kind == Some(CharKind::Word) {
                    let text: String = snapshot.text_for_range(range).collect();
                    if !text.trim().is_empty() {
                        return text;
                    }
                }
                String::new()
            }
        }
    }

    fn activate_match(
        &mut self,
        index: usize,
        matches: &[Range<Anchor>],
        cx: &mut ViewContext<Self>,
    ) {
        self.unfold_ranges(&[matches[index].clone()], false, true, cx);
        let range = self.range_for_match(&matches[index]);
        self.change_selections(Some(Autoscroll::fit()), cx, |s| {
            s.select_ranges([range]);
        })
    }

    fn select_matches(&mut self, matches: &[Self::Match], cx: &mut ViewContext<Self>) {
        self.unfold_ranges(matches, false, false, cx);
        let mut ranges = Vec::new();
        for m in matches {
            ranges.push(self.range_for_match(m))
        }
        self.change_selections(None, cx, |s| s.select_ranges(ranges));
    }
    fn replace(
        &mut self,
        identifier: &Self::Match,
        query: &SearchQuery,
        cx: &mut ViewContext<Self>,
    ) {
        let text = self.buffer.read(cx);
        let text = text.snapshot(cx);
        let text = text.text_for_range(identifier.clone()).collect::<Vec<_>>();
        let text: Cow<_> = if text.len() == 1 {
            text.first().cloned().unwrap().into()
        } else {
            let joined_chunks = text.join("");
            joined_chunks.into()
        };

        if let Some(replacement) = query.replacement_for(&text) {
            self.transact(cx, |this, cx| {
                this.edit([(identifier.clone(), Arc::from(&*replacement))], cx);
            });
        }
    }
    fn replace_all(
        &mut self,
        matches: &mut dyn Iterator<Item = &Self::Match>,
        query: &SearchQuery,
        cx: &mut ViewContext<Self>,
    ) {
        let text = self.buffer.read(cx);
        let text = text.snapshot(cx);
        let mut edits = vec![];
        for m in matches {
            let text = text.text_for_range(m.clone()).collect::<Vec<_>>();
            let text: Cow<_> = if text.len() == 1 {
                text.first().cloned().unwrap().into()
            } else {
                let joined_chunks = text.join("");
                joined_chunks.into()
            };

            if let Some(replacement) = query.replacement_for(&text) {
                edits.push((m.clone(), Arc::from(&*replacement)));
            }
        }

        if !edits.is_empty() {
            self.transact(cx, |this, cx| {
                this.edit(edits, cx);
            });
        }
    }
    fn match_index_for_direction(
        &mut self,
        matches: &[Range<Anchor>],
        current_index: usize,
        direction: Direction,
        count: usize,
        cx: &mut ViewContext<Self>,
    ) -> usize {
        let buffer = self.buffer().read(cx).snapshot(cx);
        let current_index_position = if self.selections.disjoint_anchors().len() == 1 {
            self.selections.newest_anchor().head()
        } else {
            matches[current_index].start
        };

        let mut count = count % matches.len();
        if count == 0 {
            return current_index;
        }
        match direction {
            Direction::Next => {
                if matches[current_index]
                    .start
                    .cmp(&current_index_position, &buffer)
                    .is_gt()
                {
                    count -= 1
                }

                (current_index + count) % matches.len()
            }
            Direction::Prev => {
                if matches[current_index]
                    .end
                    .cmp(&current_index_position, &buffer)
                    .is_lt()
                {
                    count -= 1;
                }

                if current_index >= count {
                    current_index - count
                } else {
                    matches.len() - (count - current_index)
                }
            }
        }
    }

    fn find_matches(
        &mut self,
        query: Arc<project::search::SearchQuery>,
        cx: &mut ViewContext<Self>,
    ) -> Task<Vec<Range<Anchor>>> {
        let buffer = self.buffer().read(cx).snapshot(cx);
        let search_within_ranges = self
            .background_highlights
            .get(&TypeId::of::<SearchWithinRange>())
            .map_or(vec![], |(_color, ranges)| {
                ranges.iter().cloned().collect::<Vec<_>>()
            });

        cx.background_executor().spawn(async move {
            let mut ranges = Vec::new();

            if let Some((_, _, excerpt_buffer)) = buffer.as_singleton() {
                let search_within_ranges = if search_within_ranges.is_empty() {
                    vec![None]
                } else {
                    search_within_ranges
                        .into_iter()
                        .map(|range| Some(range.to_offset(&buffer)))
                        .collect::<Vec<_>>()
                };

                for range in search_within_ranges {
                    let buffer = &buffer;
                    ranges.extend(
                        query
                            .search(excerpt_buffer, range.clone())
                            .await
                            .into_iter()
                            .map(|matched_range| {
                                let offset = range.clone().map(|r| r.start).unwrap_or(0);
                                buffer.anchor_after(matched_range.start + offset)
                                    ..buffer.anchor_before(matched_range.end + offset)
                            }),
                    );
                }
            } else {
                let search_within_ranges = if search_within_ranges.is_empty() {
                    vec![buffer.anchor_before(0)..buffer.anchor_after(buffer.len())]
                } else {
                    search_within_ranges
                };

                for (excerpt_id, search_buffer, search_range) in
                    buffer.excerpts_in_ranges(search_within_ranges)
                {
                    if !search_range.is_empty() {
                        ranges.extend(
                            query
                                .search(search_buffer, Some(search_range.clone()))
                                .await
                                .into_iter()
                                .map(|match_range| {
                                    let start = search_buffer
                                        .anchor_after(search_range.start + match_range.start);
                                    let end = search_buffer
                                        .anchor_before(search_range.start + match_range.end);
                                    buffer.anchor_in_excerpt(excerpt_id, start).unwrap()
                                        ..buffer.anchor_in_excerpt(excerpt_id, end).unwrap()
                                }),
                        );
                    }
                }
            };

            ranges
        })
    }

    fn active_match_index(
        &mut self,
        matches: &[Range<Anchor>],
        cx: &mut ViewContext<Self>,
    ) -> Option<usize> {
        active_match_index(
            matches,
            &self.selections.newest_anchor().head(),
            &self.buffer().read(cx).snapshot(cx),
        )
    }

    fn search_bar_visibility_changed(&mut self, _visible: bool, _cx: &mut ViewContext<Self>) {
        self.expect_bounds_change = self.last_bounds;
    }
}

pub fn active_match_index(
    ranges: &[Range<Anchor>],
    cursor: &Anchor,
    buffer: &MultiBufferSnapshot,
) -> Option<usize> {
    if ranges.is_empty() {
        None
    } else {
        match ranges.binary_search_by(|probe| {
            if probe.end.cmp(cursor, buffer).is_lt() {
                Ordering::Less
            } else if probe.start.cmp(cursor, buffer).is_gt() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            Ok(i) | Err(i) => Some(cmp::min(i, ranges.len() - 1)),
        }
    }
}

pub fn entry_label_color(selected: bool) -> Color {
    if selected {
        Color::Default
    } else {
        Color::Muted
    }
}

pub fn entry_diagnostic_aware_icon_name_and_color(
    diagnostic_severity: Option<DiagnosticSeverity>,
) -> Option<(IconName, Color)> {
    match diagnostic_severity {
        Some(DiagnosticSeverity::ERROR) => Some((IconName::X, Color::Error)),
        Some(DiagnosticSeverity::WARNING) => Some((IconName::Triangle, Color::Warning)),
        _ => None,
    }
}

pub fn entry_diagnostic_aware_icon_decoration_and_color(
    diagnostic_severity: Option<DiagnosticSeverity>,
) -> Option<(IconDecorationKind, Color)> {
    match diagnostic_severity {
        Some(DiagnosticSeverity::ERROR) => Some((IconDecorationKind::X, Color::Error)),
        Some(DiagnosticSeverity::WARNING) => Some((IconDecorationKind::Triangle, Color::Warning)),
        _ => None,
    }
}

pub fn entry_git_aware_label_color(
    git_status: Option<GitFileStatus>,
    ignored: bool,
    selected: bool,
) -> Color {
    if ignored {
        Color::Ignored
    } else {
        match git_status {
            Some(GitFileStatus::Added) => Color::Created,
            Some(GitFileStatus::Modified) => Color::Modified,
            Some(GitFileStatus::Conflict) => Color::Conflict,
            None => entry_label_color(selected),
        }
    }
}

fn path_for_buffer<'a>(
    buffer: &Model<MultiBuffer>,
    height: usize,
    include_filename: bool,
    cx: &'a AppContext,
) -> Option<Cow<'a, Path>> {
    let file = buffer.read(cx).as_singleton()?.read(cx).file()?;
    path_for_file(file.as_ref(), height, include_filename, cx)
}

fn path_for_file<'a>(
    file: &'a dyn language::File,
    mut height: usize,
    include_filename: bool,
    cx: &'a AppContext,
) -> Option<Cow<'a, Path>> {
    // Ensure we always render at least the filename.
    height += 1;

    let mut prefix = file.path().as_ref();
    while height > 0 {
        if let Some(parent) = prefix.parent() {
            prefix = parent;
            height -= 1;
        } else {
            break;
        }
    }

    // Here we could have just always used `full_path`, but that is very
    // allocation-heavy and so we try to use a `Cow<Path>` if we haven't
    // traversed all the way up to the worktree's root.
    if height > 0 {
        let full_path = file.full_path(cx);
        if include_filename {
            Some(full_path.into())
        } else {
            Some(full_path.parent()?.to_path_buf().into())
        }
    } else {
        let mut path = file.path().strip_prefix(prefix).ok()?;
        if !include_filename {
            path = path.parent()?;
        }
        Some(path.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::editor_tests::init_test;
    use fs::Fs;

    use super::*;
    use fs::MTime;
    use gpui::{AppContext, VisualTestContext};
    use language::{LanguageMatcher, TestFile};
    use project::FakeFs;
    use std::path::{Path, PathBuf};

    #[gpui::test]
    fn test_path_for_file(cx: &mut AppContext) {
        let file = TestFile {
            path: Path::new("").into(),
            root_name: String::new(),
        };
        assert_eq!(path_for_file(&file, 0, false, cx), None);
    }

    async fn deserialize_editor(
        item_id: ItemId,
        workspace_id: WorkspaceId,
        workspace: View<Workspace>,
        project: Model<Project>,
        cx: &mut VisualTestContext,
    ) -> View<Editor> {
        workspace
            .update(cx, |workspace, cx| {
                let pane = workspace.active_pane();
                pane.update(cx, |_, cx| {
                    Editor::deserialize(
                        project.clone(),
                        workspace.weak_handle(),
                        workspace_id,
                        item_id,
                        cx,
                    )
                })
            })
            .await
            .unwrap()
    }

    fn rust_language() -> Arc<language::Language> {
        Arc::new(language::Language::new(
            language::LanguageConfig {
                name: "Rust".into(),
                matcher: LanguageMatcher {
                    path_suffixes: vec!["rs".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            Some(tree_sitter_rust::LANGUAGE.into()),
        ))
    }

    #[gpui::test]
    async fn test_deserialize(cx: &mut gpui::TestAppContext) {
        init_test(cx, |_| {});

        let fs = FakeFs::new(cx.executor());
        fs.insert_file("/file.rs", Default::default()).await;

        // Test case 1: Deserialize with path and contents
        {
            let project = Project::test(fs.clone(), ["/file.rs".as_ref()], cx).await;
            let (workspace, cx) = cx.add_window_view(|cx| Workspace::test_new(project.clone(), cx));
            let workspace_id = workspace::WORKSPACE_DB.next_id().await.unwrap();
            let item_id = 1234 as ItemId;
            let mtime = fs
                .metadata(Path::new("/file.rs"))
                .await
                .unwrap()
                .unwrap()
                .mtime;

            let serialieditsync_editor = SerialieditsyncEditor {
                abs_path: Some(PathBuf::from("/file.rs")),
                contents: Some("fn main() {}".to_string()),
                language: Some("Rust".to_string()),
                mtime: Some(mtime),
            };

            DB.save_serialieditsync_editor(item_id, workspace_id, serialieditsync_editor.clone())
                .await
                .unwrap();

            let deserialieditsync =
                deserialize_editor(item_id, workspace_id, workspace, project, cx).await;

            deserialieditsync.update(cx, |editor, cx| {
                assert_eq!(editor.text(cx), "fn main() {}");
                assert!(editor.is_dirty(cx));
                assert!(!editor.has_conflict(cx));
                let buffer = editor.buffer().read(cx).as_singleton().unwrap().read(cx);
                assert!(buffer.file().is_some());
            });
        }

        // Test case 2: Deserialize with only path
        {
            let project = Project::test(fs.clone(), ["/file.rs".as_ref()], cx).await;
            let (workspace, cx) = cx.add_window_view(|cx| Workspace::test_new(project.clone(), cx));

            let workspace_id = workspace::WORKSPACE_DB.next_id().await.unwrap();

            let item_id = 5678 as ItemId;
            let serialieditsync_editor = SerialieditsyncEditor {
                abs_path: Some(PathBuf::from("/file.rs")),
                contents: None,
                language: None,
                mtime: None,
            };

            DB.save_serialieditsync_editor(item_id, workspace_id, serialieditsync_editor)
                .await
                .unwrap();

            let deserialieditsync =
                deserialize_editor(item_id, workspace_id, workspace, project, cx).await;

            deserialieditsync.update(cx, |editor, cx| {
                assert_eq!(editor.text(cx), ""); // The file should be empty as per our initial setup
                assert!(!editor.is_dirty(cx));
                assert!(!editor.has_conflict(cx));

                let buffer = editor.buffer().read(cx).as_singleton().unwrap().read(cx);
                assert!(buffer.file().is_some());
            });
        }

        // Test case 3: Deserialize with no path (untitled buffer, with content and language)
        {
            let project = Project::test(fs.clone(), ["/file.rs".as_ref()], cx).await;
            // Add Rust to the language, so that we can restore the language of the buffer
            project.update(cx, |project, _| project.languages().add(rust_language()));

            let (workspace, cx) = cx.add_window_view(|cx| Workspace::test_new(project.clone(), cx));

            let workspace_id = workspace::WORKSPACE_DB.next_id().await.unwrap();

            let item_id = 9012 as ItemId;
            let serialieditsync_editor = SerialieditsyncEditor {
                abs_path: None,
                contents: Some("hello".to_string()),
                language: Some("Rust".to_string()),
                mtime: None,
            };

            DB.save_serialieditsync_editor(item_id, workspace_id, serialieditsync_editor)
                .await
                .unwrap();

            let deserialieditsync =
                deserialize_editor(item_id, workspace_id, workspace, project, cx).await;

            deserialieditsync.update(cx, |editor, cx| {
                assert_eq!(editor.text(cx), "hello");
                assert!(editor.is_dirty(cx)); // The editor should be dirty for an untitled buffer

                let buffer = editor.buffer().read(cx).as_singleton().unwrap().read(cx);
                assert_eq!(
                    buffer.language().map(|lang| lang.name()),
                    Some("Rust".into())
                ); // Language should be set to Rust
                assert!(buffer.file().is_none()); // The buffer should not have an associated file
            });
        }

        // Test case 4: Deserialize with path, content, and old mtime
        {
            let project = Project::test(fs.clone(), ["/file.rs".as_ref()], cx).await;
            let (workspace, cx) = cx.add_window_view(|cx| Workspace::test_new(project.clone(), cx));

            let workspace_id = workspace::WORKSPACE_DB.next_id().await.unwrap();

            let item_id = 9345 as ItemId;
            let old_mtime = MTime::from_seconds_and_nanos(0, 50);
            let serialieditsync_editor = SerialieditsyncEditor {
                abs_path: Some(PathBuf::from("/file.rs")),
                contents: Some("fn main() {}".to_string()),
                language: Some("Rust".to_string()),
                mtime: Some(old_mtime),
            };

            DB.save_serialieditsync_editor(item_id, workspace_id, serialieditsync_editor)
                .await
                .unwrap();

            let deserialieditsync =
                deserialize_editor(item_id, workspace_id, workspace, project, cx).await;

            deserialieditsync.update(cx, |editor, cx| {
                assert_eq!(editor.text(cx), "fn main() {}");
                assert!(editor.has_conflict(cx)); // The editor should have a conflict
            });
        }
    }
}