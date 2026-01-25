import { Component, createSignal, For, Show, onMount } from "solid-js";
import {
  useProjectTemplates,
  type TemplateSummary,
  type ProjectTemplate,
  type TemplateCategory,
} from "../../hooks/useProjectTemplates";
import {
  HiOutlineFolder,
  HiOutlineDocumentText,
  HiOutlineArrowDownTray,
  HiOutlineArrowUpTray,
  HiOutlineX,
  HiOutlineMagnifyingGlass,
  HiOutlineViewGrid,
  HiOutlineViewList,
} from "../icons";

type ViewMode = "grid" | "list";

interface TemplateGalleryProps {
  isOpen: boolean;
  onClose: () => void;
  onTemplateApplied?: (templateId: string) => void;
  projectPath: string; // Real project path from parent
}

export const TemplateGallery: Component<TemplateGalleryProps> = (props) => {
  const templates = useProjectTemplates();

  const [viewMode, setViewMode] = createSignal<ViewMode>("grid");
  const [searchQuery, setSearchQuery] = createSignal("");
  const [selectedCategory, setSelectedCategory] =
    createSignal<TemplateCategory | "all">("all");
  const [selectedTemplate, setSelectedTemplate] =
    createSignal<ProjectTemplate | null>(null);
  const [showPreview, setShowPreview] = createSignal(false);
  const [showCreateDialog, setShowCreateDialog] = createSignal(false);
  const [newTemplateName, setNewTemplateName] = createSignal("");
  const [newTemplateCategory, setNewTemplateCategory] =
    createSignal<TemplateCategory>("General");
  const [newTemplateDesc, setNewTemplateDesc] = createSignal("");

  onMount(() => {
    templates.listTemplates();
  });

  const categories: Array<{ value: TemplateCategory | "all"; label: string }> =
    [
      { value: "all", label: "All Templates" },
      { value: "Mobile", label: "Mobile Forensics" },
      { value: "Computer", label: "Computer Forensics" },
      { value: "Network", label: "Network Forensics" },
      { value: "Cloud", label: "Cloud Forensics" },
      { value: "IncidentResponse", label: "Incident Response" },
      { value: "Memory", label: "Memory Analysis" },
      { value: "Malware", label: "Malware Analysis" },
      { value: "EDiscovery", label: "E-Discovery" },
      { value: "General", label: "General" },
      { value: "Custom", label: "Custom" },
    ];

  const filteredTemplates = () => {
    let result = templates.templates();

    // Filter by category
    if (selectedCategory() !== "all") {
      result = result.filter((t) => t.category === selectedCategory());
    }

    // Filter by search query
    const query = searchQuery().toLowerCase();
    if (query) {
      result = result.filter(
        (t) =>
          t.name.toLowerCase().includes(query) ||
          t.description.toLowerCase().includes(query)
      );
    }

    return result;
  };

  const handleApplyTemplate = async (templateId: string) => {
    if (!props.projectPath) {
      console.error("No project path provided");
      return;
    }
    const success = await templates.applyTemplate(props.projectPath, templateId);
    if (success) {
      props.onTemplateApplied?.(templateId);
      props.onClose();
    }
  };

  const handlePreview = async (templateSummary: TemplateSummary) => {
    const fullTemplate = await templates.getTemplate(templateSummary.id);
    if (fullTemplate) {
      setSelectedTemplate(fullTemplate);
      setShowPreview(true);
    }
  };

  const handleCreateFromProject = async () => {
    const name = newTemplateName();
    const category = newTemplateCategory();
    const description = newTemplateDesc();

    if (!name || !props.projectPath) {
      console.error("Missing template name or project path");
      return;
    }

    const templateId = await templates.createFromProject(
      props.projectPath,
      name,
      category,
      description
    );
    if (templateId) {
      setShowCreateDialog(false);
      setNewTemplateName("");
      setNewTemplateDesc("");
      templates.listTemplates(); // Refresh
    }
  };

  const handleExportTemplate = async (templateId: string) => {
    await templates.exportTemplate(templateId, "/export/path/template.json");
  };

  const handleImportTemplate = async () => {
    const success = await templates.importTemplate("/import/path/template.json");
    if (success) {
      templates.listTemplates(); // Refresh
    }
  };

  const getCategoryColor = (category: TemplateCategory) => {
    switch (category) {
      case "Mobile":
        return "text-type-ufed";
      case "Computer":
        return "text-type-e01";
      case "Network":
        return "text-accent";
      case "Cloud":
        return "text-info";
      case "IncidentResponse":
        return "text-warning";
      case "Memory":
        return "text-type-raw";
      case "Malware":
        return "text-error";
      case "EDiscovery":
        return "text-type-ad1";
      case "General":
        return "text-success";
      case "Custom":
        return "text-txt-secondary";
      default:
        return "text-txt-secondary";
    }
  };

  const getCategoryLabel = (category: TemplateCategory): string => {
    const labels: Record<TemplateCategory, string> = {
      Mobile: "Mobile Forensics",
      Computer: "Computer Forensics",
      Network: "Network Forensics",
      Cloud: "Cloud Forensics",
      IncidentResponse: "Incident Response",
      Memory: "Memory Analysis",
      Malware: "Malware Analysis",
      EDiscovery: "E-Discovery",
      General: "General",
      Custom: "Custom",
    };
    return labels[category] || category;
  };

  if (!props.isOpen) return null;

  return (
    <>
      {/* Modal Backdrop */}
      <div
        class="fixed inset-0 bg-black/50 z-modal-backdrop"
        onClick={props.onClose}
      />

      {/* Modal Content */}
      <div class="fixed inset-0 z-modal flex items-center justify-center p-4">
        <div class="bg-bg-panel rounded-lg border border-border w-full max-w-6xl max-h-[90vh] flex flex-col">
          {/* Header */}
          <div class="flex items-center justify-between p-4 border-b border-border">
            <div class="flex items-center gap-3">
              <HiOutlineFolder class="w-icon-lg h-icon-lg text-accent" />
              <h2 class="text-lg font-semibold text-txt">
                Template Gallery
              </h2>
            </div>
            <button
              onClick={props.onClose}
              class="p-2 hover:bg-bg-hover rounded-md text-txt-secondary hover:text-txt"
            >
              <HiOutlineX class="w-icon-base h-icon-base" />
            </button>
          </div>

          {/* Toolbar */}
          <div class="p-4 border-b border-border flex flex-wrap items-center gap-3">
            {/* Search */}
            <div class="flex-1 min-w-[200px] flex items-center gap-2 bg-bg rounded-md px-3 py-2 border border-border">
              <HiOutlineMagnifyingGlass class="w-icon-sm h-icon-sm text-txt-secondary" />
              <input
                type="text"
                placeholder="Search templates..."
                value={searchQuery()}
                onInput={(e) => setSearchQuery(e.currentTarget.value)}
                class="flex-1 bg-transparent text-txt placeholder-txt-muted outline-none"
              />
            </div>

            {/* Category Filter */}
            <select
              value={selectedCategory()}
              onChange={(e) =>
                setSelectedCategory(
                  e.currentTarget.value as TemplateCategory | "all"
                )
              }
              class="input"
            >
              <For each={categories}>
                {(cat) => <option value={cat.value}>{cat.label}</option>}
              </For>
            </select>

            {/* View Mode Toggle */}
            <div class="flex gap-1 bg-bg border border-border rounded-md p-1">
              <button
                onClick={() => setViewMode("grid")}
                class={`p-2 rounded ${
                  viewMode() === "grid"
                    ? "bg-bg-hover text-accent"
                    : "text-txt-secondary hover:text-txt"
                }`}
              >
                <HiOutlineViewGrid class="w-icon-sm h-icon-sm" />
              </button>
              <button
                onClick={() => setViewMode("list")}
                class={`p-2 rounded ${
                  viewMode() === "list"
                    ? "bg-bg-hover text-accent"
                    : "text-txt-secondary hover:text-txt"
                }`}
              >
                <HiOutlineViewList class="w-icon-sm h-icon-sm" />
              </button>
            </div>

            {/* Actions */}
            <button
              onClick={() => setShowCreateDialog(true)}
              class="btn btn-primary"
            >
              <HiOutlineDocumentText class="w-icon-sm h-icon-sm" />
              Create from Project
            </button>
            <button
              onClick={handleImportTemplate}
              class="btn btn-secondary"
            >
              <HiOutlineArrowUpTray class="w-icon-sm h-icon-sm" />
              Import
            </button>
          </div>

          {/* Content */}
          <div class="flex-1 overflow-auto p-4">
            <Show
              when={!templates.loading()}
              fallback={
                <div class="flex items-center justify-center h-full text-txt-muted">
                  Loading templates...
                </div>
              }
            >
              <Show
                when={filteredTemplates().length > 0}
                fallback={
                  <div class="flex items-center justify-center h-full text-txt-muted">
                    No templates found
                  </div>
                }
              >
                <Show
                  when={viewMode() === "grid"}
                  fallback={
                    <div class="space-y-2">
                      <For each={filteredTemplates()}>
                        {(template) => (
                          <div class="bg-bg border border-border rounded-md p-4 hover:bg-bg-hover flex items-center gap-4">
                            <div class="flex-1">
                              <div class="flex items-center gap-2 mb-1">
                                <h3 class="font-medium text-txt">
                                  {template.name}
                                </h3>
                                <Show when={templates.isBuiltinTemplate(template.id)}>
                                  <span class="px-2 py-0.5 bg-accent/20 text-accent text-xs rounded">
                                    Built-in
                                  </span>
                                </Show>
                                <span
                                  class={`px-2 py-0.5 text-xs rounded ${getCategoryColor(
                                    template.category
                                  )}`}
                                >
                                  {getCategoryLabel(template.category)}
                                </span>
                              </div>
                              <p class="text-sm text-txt-secondary">
                                {template.description}
                              </p>
                              <div class="flex items-center gap-4 mt-2 text-xs text-txt-muted">
                                <span class="flex items-center gap-1">
                                  Used {template.usage_count} times
                                </span>
                                <span class="flex items-center gap-1">
                                  {template.tags.length} tags
                                </span>
                              </div>
                            </div>
                            <div class="flex items-center gap-2">
                              <button
                                onClick={() => handlePreview(template)}
                                class="px-3 py-1.5 bg-bg-secondary hover:bg-bg-hover text-txt text-sm rounded border border-border"
                              >
                                Preview
                              </button>
                              <button
                                onClick={() => handleApplyTemplate(template.id)}
                                class="px-3 py-1.5 bg-accent hover:bg-accent-hover text-white text-sm rounded"
                              >
                                Apply
                              </button>
                              <Show when={!templates.isBuiltinTemplate(template.id)}>
                                <button
                                  onClick={() =>
                                    handleExportTemplate(template.id)
                                  }
                                  class="p-1.5 hover:bg-bg-hover text-txt-secondary hover:text-txt rounded"
                                >
                                  <HiOutlineArrowDownTray class="w-icon-sm h-icon-sm" />
                                </button>
                              </Show>
                            </div>
                          </div>
                        )}
                      </For>
                    </div>
                  }
                >
                  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    <For each={filteredTemplates()}>
                      {(template) => (
                        <div class="bg-bg border border-border rounded-md p-4 hover:bg-bg-hover flex flex-col">
                          <div class="flex items-start justify-between mb-2">
                            <h3 class="font-medium text-txt">
                              {template.name}
                            </h3>
                            <Show when={templates.isBuiltinTemplate(template.id)}>
                              <span class="px-2 py-0.5 bg-accent/20 text-accent text-xs rounded shrink-0">
                                Built-in
                              </span>
                            </Show>
                          </div>
                          <span
                            class={`inline-block px-2 py-0.5 text-xs rounded mb-2 ${getCategoryColor(
                              template.category
                            )}`}
                          >
                            {getCategoryLabel(template.category)}
                          </span>
                          <p class="text-sm text-txt-secondary mb-3 flex-1">
                            {template.description}
                          </p>
                          <div class="flex items-center gap-3 text-xs text-txt-muted mb-3">
                            <span>Used {template.usage_count} times</span>
                            <span>{template.tags.length} tags</span>
                          </div>
                          <div class="flex gap-2">
                            <button
                              onClick={() => handlePreview(template)}
                              class="flex-1 px-3 py-1.5 bg-bg-secondary hover:bg-bg-hover text-txt text-sm rounded border border-border"
                            >
                              Preview
                            </button>
                            <button
                              onClick={() => handleApplyTemplate(template.id)}
                              class="flex-1 px-3 py-1.5 bg-accent hover:bg-accent-hover text-white text-sm rounded"
                            >
                              Apply
                            </button>
                            <Show when={!templates.isBuiltinTemplate(template.id)}>
                              <button
                                onClick={() => handleExportTemplate(template.id)}
                                class="p-1.5 hover:bg-bg-hover text-txt-secondary hover:text-txt rounded border border-border"
                              >
                                <HiOutlineArrowDownTray class="w-icon-sm h-icon-sm" />
                              </button>
                            </Show>
                          </div>
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
              </Show>
            </Show>
          </div>
        </div>
      </div>

      {/* Preview Modal */}
      <Show when={showPreview() && selectedTemplate()}>
        {(template) => (
          <>
            <div
              class="fixed inset-0 bg-black/50 z-modal-backdrop"
              onClick={() => setShowPreview(false)}
            />
            <div class="fixed inset-0 z-modal flex items-center justify-center p-4">
              <div class="bg-bg-panel rounded-lg border border-border w-full max-w-4xl max-h-[80vh] flex flex-col">
                <div class="flex items-center justify-between p-4 border-b border-border">
                  <h3 class="text-lg font-semibold text-txt">
                    {template().name}
                  </h3>
                  <button
                    onClick={() => setShowPreview(false)}
                    class="p-2 hover:bg-bg-hover rounded-md text-txt-secondary hover:text-txt"
                  >
                    <HiOutlineX class="w-icon-base h-icon-base" />
                  </button>
                </div>
                <div class="flex-1 overflow-auto p-4">
                  <div class="space-y-4">
                    <div>
                      <h4 class="text-sm font-medium text-txt mb-2">
                        Description
                      </h4>
                      <p class="text-sm text-txt-secondary">
                        {template().description}
                      </p>
                    </div>
                    <Show when={template().bookmarks.length > 0}>
                      <div>
                        <h4 class="text-sm font-medium text-txt mb-2">
                          Bookmarks ({template().bookmarks.length})
                        </h4>
                        <div class="text-sm text-txt-muted">
                          Template includes {template().bookmarks.length}{" "}
                          bookmarks
                        </div>
                      </div>
                    </Show>
                    <Show when={template().notes.length > 0}>
                      <div>
                        <h4 class="text-sm font-medium text-txt mb-2">
                          Notes ({template().notes.length})
                        </h4>
                        <div class="text-sm text-txt-muted">
                          Template includes {template().notes.length} notes
                        </div>
                      </div>
                    </Show>
                  </div>
                </div>
                <div class="modal-footer justify-end">
                  <button
                    onClick={() => setShowPreview(false)}
                    class="btn btn-secondary"
                  >
                    Close
                  </button>
                  <button
                    onClick={() => {
                      handleApplyTemplate(template().id);
                      setShowPreview(false);
                    }}
                    class="btn btn-primary"
                  >
                    Apply Template
                  </button>
                </div>
              </div>
            </div>
          </>
        )}
      </Show>

      {/* Create Template Dialog */}
      <Show when={showCreateDialog()}>
        <div class="modal-overlay" onClick={() => setShowCreateDialog(false)}>
          <div class="modal-content max-w-md" onClick={(e) => e.stopPropagation()}>
            <div class="modal-header">
              <h3 class="text-lg font-semibold text-txt">
                Create Template from Project
              </h3>
            </div>
            <div class="modal-body space-y-4">
              <div class="form-group">
                <label class="label">
                  Template Name
                </label>
                <input
                  type="text"
                  value={newTemplateName()}
                  onInput={(e) => setNewTemplateName(e.currentTarget.value)}
                  class="input"
                  placeholder="Enter template name"
                />
              </div>
              <div class="form-group">
                <label class="label">
                  Category
                </label>
                <select
                  value={newTemplateCategory()}
                  onChange={(e) =>
                    setNewTemplateCategory(
                      e.currentTarget.value as TemplateCategory
                    )
                  }
                  class="input"
                >
                  <option value="Mobile">Mobile Forensics</option>
                  <option value="Computer">Computer Forensics</option>
                  <option value="Network">Network Forensics</option>
                  <option value="Cloud">Cloud Forensics</option>
                  <option value="IncidentResponse">Incident Response</option>
                  <option value="Memory">Memory Analysis</option>
                  <option value="Malware">Malware Analysis</option>
                  <option value="EDiscovery">E-Discovery</option>
                  <option value="General">General</option>
                  <option value="Custom">Custom</option>
                </select>
              </div>
              <div class="form-group">
                <label class="label">
                  Description (Optional)
                </label>
                <textarea
                  value={newTemplateDesc()}
                  onInput={(e) => setNewTemplateDesc(e.currentTarget.value)}
                  rows={3}
                  class="textarea"
                  placeholder="Describe this template..."
                />
              </div>
            </div>
            <div class="modal-footer justify-end">
              <button
                onClick={() => setShowCreateDialog(false)}
                class="btn btn-secondary"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateFromProject}
                disabled={!newTemplateName()}
                class="btn btn-primary"
              >
                Create Template
              </button>
            </div>
          </div>
        </div>
      </Show>
    </>
  );
};
