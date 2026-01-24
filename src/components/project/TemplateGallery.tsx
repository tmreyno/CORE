import { Component, createSignal, For, Show, onMount } from "solid-js";
import {
  useProjectTemplates,
  type TemplateSummary,
  type ProjectTemplate,
} from "../../hooks/useProjectTemplates";
import {
  HiOutlineFolder,
  HiOutlineDocumentText,
  HiOutlineBookmark,
  HiOutlineArrowDownTray,
  HiOutlineArrowUpTray,
  HiOutlineX,
  HiOutlineMagnifyingGlass,
  HiOutlineViewGrid,
  HiOutlineViewList,
} from "../icons";

type ViewMode = "grid" | "list";
type TemplateCategory =
  | "investigation"
  | "malware_analysis"
  | "incident_response"
  | "mobile_forensics"
  | "data_breach"
  | "custom";

interface TemplateGalleryProps {
  isOpen: boolean;
  onClose: () => void;
  onTemplateApplied?: (templateId: string) => void;
  projectPath: string; // Real project path from parent
}

const TemplateGallery: Component<TemplateGalleryProps> = (props) => {
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
    createSignal<TemplateCategory>("investigation");
  const [newTemplateDesc, setNewTemplateDesc] = createSignal("");

  onMount(() => {
    templates.listTemplates();
  });

  const categories: Array<{ value: TemplateCategory | "all"; label: string }> =
    [
      { value: "all", label: "All Templates" },
      { value: "investigation", label: "Investigation" },
      { value: "malware_analysis", label: "Malware Analysis" },
      { value: "incident_response", label: "Incident Response" },
      { value: "mobile_forensics", label: "Mobile Forensics" },
      { value: "data_breach", label: "Data Breach" },
      { value: "custom", label: "Custom" },
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

  const getCategoryColor = (category: string) => {
    switch (category) {
      case "investigation":
        return "text-type-ad1";
      case "malware_analysis":
        return "text-error";
      case "incident_response":
        return "text-warning";
      case "mobile_forensics":
        return "text-type-e01";
      case "data_breach":
        return "text-type-ufed";
      case "custom":
        return "text-accent";
      default:
        return "text-txt-secondary";
    }
  };

  if (!props.isOpen) return null;

  return (
    <>
      {/* Modal Backdrop */}
      <div
        className="fixed inset-0 bg-black/50 z-modal-backdrop"
        onClick={props.onClose}
      />

      {/* Modal Content */}
      <div className="fixed inset-0 z-modal flex items-center justify-center p-4">
        <div className="bg-bg-panel rounded-lg border border-border w-full max-w-6xl max-h-[90vh] flex flex-col">
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-border">
            <div className="flex items-center gap-3">
              <HiOutlineFolder class="w-icon-lg h-icon-lg text-accent" />
              <h2 className="text-lg font-semibold text-txt">
                Template Gallery
              </h2>
            </div>
            <button
              onClick={props.onClose}
              className="p-2 hover:bg-bg-hover rounded-md text-txt-secondary hover:text-txt"
            >
              <HiOutlineX class="w-icon-base h-icon-base" />
            </button>
          </div>

          {/* Toolbar */}
          <div className="p-4 border-b border-border flex flex-wrap items-center gap-3">
            {/* Search */}
            <div className="flex-1 min-w-[200px] flex items-center gap-2 bg-bg rounded-md px-3 py-2 border border-border">
              <HiOutlineMagnifyingGlass class="w-icon-sm h-icon-sm text-txt-secondary" />
              <input
                type="text"
                placeholder="Search templates..."
                value={searchQuery()}
                onInput={(e) => setSearchQuery(e.currentTarget.value)}
                className="flex-1 bg-transparent text-txt placeholder-txt-muted outline-none"
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
              className="px-3 py-2 bg-bg border border-border rounded-md text-txt"
            >
              <For each={categories}>
                {(cat) => <option value={cat.value}>{cat.label}</option>}
              </For>
            </select>

            {/* View Mode Toggle */}
            <div className="flex gap-1 bg-bg border border-border rounded-md p-1">
              <button
                onClick={() => setViewMode("grid")}
                className={`p-2 rounded ${
                  viewMode() === "grid"
                    ? "bg-bg-hover text-accent"
                    : "text-txt-secondary hover:text-txt"
                }`}
              >
                <HiOutlineViewGrid class="w-icon-sm h-icon-sm" />
              </button>
              <button
                onClick={() => setViewMode("list")}
                className={`p-2 rounded ${
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
              className="px-3 py-2 bg-accent hover:bg-accent-hover text-white rounded-md flex items-center gap-2"
            >
              <HiOutlineDocumentText class="w-icon-sm h-icon-sm" />
              Create from Project
            </button>
            <button
              onClick={handleImportTemplate}
              className="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md flex items-center gap-2 border border-border"
            >
              <HiOutlineArrowUpTray class="w-icon-sm h-icon-sm" />
              Import
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-auto p-4">
            <Show
              when={!templates.loading()}
              fallback={
                <div className="flex items-center justify-center h-full text-txt-muted">
                  Loading templates...
                </div>
              }
            >
              <Show
                when={filteredTemplates().length > 0}
                fallback={
                  <div className="flex items-center justify-center h-full text-txt-muted">
                    No templates found
                  </div>
                }
              >
                <Show
                  when={viewMode() === "grid"}
                  fallback={
                    <div className="space-y-2">
                      <For each={filteredTemplates()}>
                        {(template) => (
                          <div className="bg-bg border border-border rounded-md p-4 hover:bg-bg-hover flex items-center gap-4">
                            <div className="flex-1">
                              <div className="flex items-center gap-2 mb-1">
                                <h3 className="font-medium text-txt">
                                  {template.name}
                                </h3>
                                <Show when={template.is_builtin}>
                                  <span className="px-2 py-0.5 bg-accent/20 text-accent text-xs rounded">
                                    Built-in
                                  </span>
                                </Show>
                                <span
                                  className={`px-2 py-0.5 text-xs rounded ${getCategoryColor(
                                    template.category
                                  )}`}
                                >
                                  {template.category}
                                </span>
                              </div>
                              <p className="text-sm text-txt-secondary">
                                {template.description}
                              </p>
                              <div className="flex items-center gap-4 mt-2 text-xs text-txt-muted">
                                <span className="flex items-center gap-1">
                                  <HiOutlineBookmark class="w-3 h-3" />
                                  {template.bookmark_count} bookmarks
                                </span>
                                <span className="flex items-center gap-1">
                                  <HiOutlineDocumentText class="w-3 h-3" />
                                  {template.note_count} notes
                                </span>
                              </div>
                            </div>
                            <div className="flex items-center gap-2">
                              <button
                                onClick={() => handlePreview(template)}
                                className="px-3 py-1.5 bg-bg-secondary hover:bg-bg-hover text-txt text-sm rounded border border-border"
                              >
                                Preview
                              </button>
                              <button
                                onClick={() => handleApplyTemplate(template.id)}
                                className="px-3 py-1.5 bg-accent hover:bg-accent-hover text-white text-sm rounded"
                              >
                                Apply
                              </button>
                              <Show when={!template.is_builtin}>
                                <button
                                  onClick={() =>
                                    handleExportTemplate(template.id)
                                  }
                                  className="p-1.5 hover:bg-bg-hover text-txt-secondary hover:text-txt rounded"
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
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    <For each={filteredTemplates()}>
                      {(template) => (
                        <div className="bg-bg border border-border rounded-md p-4 hover:bg-bg-hover flex flex-col">
                          <div className="flex items-start justify-between mb-2">
                            <h3 className="font-medium text-txt">
                              {template.name}
                            </h3>
                            <Show when={template.is_builtin}>
                              <span className="px-2 py-0.5 bg-accent/20 text-accent text-xs rounded shrink-0">
                                Built-in
                              </span>
                            </Show>
                          </div>
                          <span
                            className={`inline-block px-2 py-0.5 text-xs rounded mb-2 ${getCategoryColor(
                              template.category
                            )}`}
                          >
                            {template.category}
                          </span>
                          <p className="text-sm text-txt-secondary mb-3 flex-1">
                            {template.description}
                          </p>
                          <div className="flex items-center gap-3 text-xs text-txt-muted mb-3">
                            <span className="flex items-center gap-1">
                              <HiOutlineBookmark class="w-3 h-3" />
                              {template.bookmark_count}
                            </span>
                            <span className="flex items-center gap-1">
                              <HiOutlineDocumentText class="w-3 h-3" />
                              {template.note_count}
                            </span>
                          </div>
                          <div className="flex gap-2">
                            <button
                              onClick={() => handlePreview(template)}
                              className="flex-1 px-3 py-1.5 bg-bg-secondary hover:bg-bg-hover text-txt text-sm rounded border border-border"
                            >
                              Preview
                            </button>
                            <button
                              onClick={() => handleApplyTemplate(template.id)}
                              className="flex-1 px-3 py-1.5 bg-accent hover:bg-accent-hover text-white text-sm rounded"
                            >
                              Apply
                            </button>
                            <Show when={!template.is_builtin}>
                              <button
                                onClick={() => handleExportTemplate(template.id)}
                                className="p-1.5 hover:bg-bg-hover text-txt-secondary hover:text-txt rounded border border-border"
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
              className="fixed inset-0 bg-black/50 z-modal-backdrop"
              onClick={() => setShowPreview(false)}
            />
            <div className="fixed inset-0 z-modal flex items-center justify-center p-4">
              <div className="bg-bg-panel rounded-lg border border-border w-full max-w-4xl max-h-[80vh] flex flex-col">
                <div className="flex items-center justify-between p-4 border-b border-border">
                  <h3 className="text-lg font-semibold text-txt">
                    {template().name}
                  </h3>
                  <button
                    onClick={() => setShowPreview(false)}
                    className="p-2 hover:bg-bg-hover rounded-md text-txt-secondary hover:text-txt"
                  >
                    <HiOutlineX class="w-icon-base h-icon-base" />
                  </button>
                </div>
                <div className="flex-1 overflow-auto p-4">
                  <div className="space-y-4">
                    <div>
                      <h4 className="text-sm font-medium text-txt mb-2">
                        Description
                      </h4>
                      <p className="text-sm text-txt-secondary">
                        {template().description}
                      </p>
                    </div>
                    <Show when={template().bookmarks.length > 0}>
                      <div>
                        <h4 className="text-sm font-medium text-txt mb-2">
                          Bookmarks ({template().bookmarks.length})
                        </h4>
                        <div className="text-sm text-txt-muted">
                          Template includes {template().bookmarks.length}{" "}
                          bookmarks
                        </div>
                      </div>
                    </Show>
                    <Show when={template().notes.length > 0}>
                      <div>
                        <h4 className="text-sm font-medium text-txt mb-2">
                          Notes ({template().notes.length})
                        </h4>
                        <div className="text-sm text-txt-muted">
                          Template includes {template().notes.length} notes
                        </div>
                      </div>
                    </Show>
                  </div>
                </div>
                <div className="p-4 border-t border-border flex justify-end gap-2">
                  <button
                    onClick={() => setShowPreview(false)}
                    className="px-4 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md border border-border"
                  >
                    Close
                  </button>
                  <button
                    onClick={() => {
                      handleApplyTemplate(template().id);
                      setShowPreview(false);
                    }}
                    className="px-4 py-2 bg-accent hover:bg-accent-hover text-white rounded-md"
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
        <div
          className="fixed inset-0 bg-black/50 z-modal-backdrop"
          onClick={() => setShowCreateDialog(false)}
        />
        <div className="fixed inset-0 z-modal flex items-center justify-center p-4">
          <div className="bg-bg-panel rounded-lg border border-border w-full max-w-md">
            <div className="p-4 border-b border-border">
              <h3 className="text-lg font-semibold text-txt">
                Create Template from Project
              </h3>
            </div>
            <div className="p-4 space-y-4">
              <div>
                <label className="block text-sm font-medium text-txt mb-1">
                  Template Name
                </label>
                <input
                  type="text"
                  value={newTemplateName()}
                  onInput={(e) => setNewTemplateName(e.currentTarget.value)}
                  className="w-full px-3 py-2 bg-bg border border-border rounded-md text-txt placeholder-txt-muted"
                  placeholder="Enter template name"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-txt mb-1">
                  Category
                </label>
                <select
                  value={newTemplateCategory()}
                  onChange={(e) =>
                    setNewTemplateCategory(
                      e.currentTarget.value as TemplateCategory
                    )
                  }
                  className="w-full px-3 py-2 bg-bg border border-border rounded-md text-txt"
                >
                  <option value="investigation">Investigation</option>
                  <option value="malware_analysis">Malware Analysis</option>
                  <option value="incident_response">Incident Response</option>
                  <option value="mobile_forensics">Mobile Forensics</option>
                  <option value="data_breach">Data Breach</option>
                  <option value="custom">Custom</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-txt mb-1">
                  Description (Optional)
                </label>
                <textarea
                  value={newTemplateDesc()}
                  onInput={(e) => setNewTemplateDesc(e.currentTarget.value)}
                  rows={3}
                  className="w-full px-3 py-2 bg-bg border border-border rounded-md text-txt placeholder-txt-muted resize-none"
                  placeholder="Describe this template..."
                />
              </div>
            </div>
            <div className="p-4 border-t border-border flex justify-end gap-2">
              <button
                onClick={() => setShowCreateDialog(false)}
                className="px-4 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md border border-border"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateFromProject}
                disabled={!newTemplateName()}
                className="px-4 py-2 bg-accent hover:bg-accent-hover text-white rounded-md disabled:opacity-50 disabled:cursor-not-allowed"
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

export default TemplateGallery;
