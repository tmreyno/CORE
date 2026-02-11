import { Component, createSignal, For, Show, onMount } from "solid-js";
import {
  useProjectTemplates,
  type TemplateSummary,
  type ProjectTemplate,
  type TemplateCategory,
} from "../../hooks/useProjectTemplates";
import {
  HiOutlineFolder,
  HiOutlineX,
} from "../icons";
import { TemplateCard } from "./template-gallery/TemplateCard";
import { TemplateListItem } from "./template-gallery/TemplateListItem";
import { TemplatePreviewModal } from "./template-gallery/TemplatePreviewModal";
import { CreateTemplateDialog } from "./template-gallery/CreateTemplateDialog";
import { TemplateFilters } from "./template-gallery/TemplateFilters";
import { logger } from "../../utils/logger";
const log = logger.scope("TemplateGallery");

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
      log.error("No project path provided");
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
      log.error("Missing template name or project path");
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
          <div class="p-4 border-b border-border">
            <TemplateFilters
              searchQuery={searchQuery}
              setSearchQuery={setSearchQuery}
              selectedCategory={selectedCategory}
              setSelectedCategory={setSelectedCategory}
              viewMode={viewMode}
              setViewMode={setViewMode}
              categories={categories}
              onImport={handleImportTemplate}
              onCreateFromProject={() => setShowCreateDialog(true)}
            />
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
                          <TemplateListItem
                            template={template}
                            isBuiltin={templates.isBuiltinTemplate(template.id)}
                            onPreview={handlePreview}
                            onApply={handleApplyTemplate}
                            onExport={handleExportTemplate}
                            getCategoryColor={getCategoryColor}
                            getCategoryLabel={getCategoryLabel}
                          />
                        )}
                      </For>
                    </div>
                  }
                >
                  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    <For each={filteredTemplates()}>
                      {(template) => (
                        <TemplateCard
                          template={template}
                          isBuiltin={templates.isBuiltinTemplate(template.id)}
                          onPreview={handlePreview}
                          onApply={handleApplyTemplate}
                          onExport={handleExportTemplate}
                          getCategoryColor={getCategoryColor}
                          getCategoryLabel={getCategoryLabel}
                        />
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
          <TemplatePreviewModal
            template={template()}
            onClose={() => setShowPreview(false)}
            onApply={handleApplyTemplate}
          />
        )}
      </Show>

      {/* Create Template Dialog */}
      <Show when={showCreateDialog()}>
        <CreateTemplateDialog
          name={newTemplateName}
          setName={setNewTemplateName}
          category={newTemplateCategory}
          setCategory={setNewTemplateCategory}
          description={newTemplateDesc}
          setDescription={setNewTemplateDesc}
          onClose={() => setShowCreateDialog(false)}
          onCreate={handleCreateFromProject}
        />
      </Show>
    </>
  );
};
