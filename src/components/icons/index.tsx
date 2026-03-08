// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Centralized Heroicons Outline components for consistent iconography
 * Using solid-icons library with Heroicons outline style
 */

import { JSX, Component, splitProps } from "solid-js";
import { getExtension } from "../../utils";

// Import all icons from solid-icons/hi first, then re-export
// This pattern works better with Vite's dependency pre-bundling
import {
  // Navigation & Actions
  HiOutlineArrowLeft,
  HiOutlineArrowRight,
  HiOutlineArrowUp,
  HiOutlineArrowDown,
  HiOutlineArrowPath,
  HiOutlineArrowUturnLeft,
  HiOutlineArrowUturnRight,
  HiOutlineArrowTopRightOnSquare,
  HiOutlineArrowDownTray,
  HiOutlineArrowUpTray,
  HiOutlineArrowRightOnRectangle,
  
  // Files & Folders
  HiOutlineFolder,
  HiOutlineFolderOpen,
  HiOutlineFolderPlus,
  HiOutlineFolderMinus,
  HiOutlineFolderArrowDown,
  HiOutlineDocument,
  HiOutlineDocumentText,
  HiOutlineDocumentDuplicate,
  HiOutlineDocumentArrowDown,
  HiOutlineDocumentArrowUp,
  HiOutlineDocumentMagnifyingGlass,
  HiOutlineDocumentCheck,
  HiOutlineDocumentPlus,
  HiOutlineDocumentMinus,
  HiOutlineArchiveBox,
  HiOutlineArchiveBoxArrowDown,
  HiOutlineClipboard,
  HiOutlineClipboardDocument,
  HiOutlineClipboardDocumentList,
  HiOutlineClipboardDocumentCheck,
  HiOutlineInbox,
  
  // UI Controls
  HiOutlineChevronDown,
  HiOutlineChevronUp,
  HiOutlineChevronLeft,
  HiOutlineChevronRight,
  HiOutlineChevronDoubleDown,
  HiOutlineChevronDoubleUp,
  HiOutlineChevronDoubleLeft,
  HiOutlineChevronDoubleRight,
  HiOutlineBars3,
  HiOutlineBars4,
  HiOutlineBars3BottomLeft,
  HiOutlineBars3BottomRight,
  HiOutlineEllipsisHorizontal,
  HiOutlineEllipsisVertical,
  HiOutlineXMark,
  HiOutlinePlus,
  HiOutlineMinus,
  HiOutlineCheck,
  HiOutlineCheckCircle,
  HiOutlineCheckBadge,
  HiOutlineXCircle,
  HiOutlinePlusCircle,
  HiOutlineMinusCircle,
  
  // Search & View
  HiOutlineMagnifyingGlass,
  HiOutlineMagnifyingGlassPlus,
  HiOutlineMagnifyingGlassMinus,
  HiOutlineMagnifyingGlassCircle,
  HiOutlineEye,
  HiOutlineEyeSlash,
  HiOutlineFunnel,
  HiOutlineAdjustmentsHorizontal,
  HiOutlineAdjustmentsVertical,
  HiOutlineViewColumns,
  HiOutlineSquares2x2,
  HiOutlineListBullet,
  HiOutlineTableCells,
  HiOutlineChartBar,
  HiOutlineChartPie,
  HiOutlineChartBarSquare,
  HiOutlineArrowsRightLeft,
  
  // Status & Alerts
  HiOutlineExclamationTriangle,
  HiOutlineExclamationCircle,
  HiOutlineInformationCircle,
  HiOutlineQuestionMarkCircle,
  HiOutlineBellAlert,
  HiOutlineBell,
  HiOutlineShieldCheck,
  HiOutlineShieldExclamation,
  
  // Data & Database
  HiOutlineCircleStack,
  HiOutlineServerStack,
  HiOutlineServer,
  HiOutlineCpuChip,
  HiOutlineCommandLine,
  HiOutlineCodeBracket,
  HiOutlineCodeBracketSquare,
  HiOutlineFingerPrint,
  HiOutlineKey,
  HiOutlineLockClosed,
  HiOutlineLockOpen,
  
  // Media & Content
  HiOutlinePhoto,
  HiOutlineFilm,
  HiOutlineMusicalNote,
  HiOutlineVideoCamera,
  HiOutlineCamera,
  HiOutlineMicrophone,
  HiOutlineSpeakerWave,
  HiOutlineSpeakerXMark,
  
  // Communication
  HiOutlineEnvelope,
  HiOutlineEnvelopeOpen,
  HiOutlineChatBubbleLeft,
  HiOutlineChatBubbleLeftRight,
  HiOutlineChatBubbleBottomCenter,
  HiOutlineChatBubbleOvalLeft,
  HiOutlinePhone,
  
  // Time & Calendar
  HiOutlineClock,
  HiOutlineCalendar,
  HiOutlineCalendarDays,
  
  // Settings & Tools
  HiOutlineCog6Tooth,
  HiOutlineCog8Tooth,
  HiOutlineWrench,
  HiOutlineWrenchScrewdriver,
  HiOutlinePaintBrush,
  HiOutlineSparkles,
  HiOutlineBolt,
  HiOutlineFire,
  HiOutlineBeaker,
  
  // Devices
  HiOutlineDevicePhoneMobile,
  HiOutlineDeviceTablet,
  HiOutlineComputerDesktop,
  HiOutlineTv,
  HiOutlinePrinter,
  HiOutlineWifi,
  HiOutlineSignal,
  
  // Storage
  HiOutlineCloudArrowDown,
  HiOutlineCloudArrowUp,
  HiOutlineCloud,
  
  // People & Users
  HiOutlineUser,
  HiOutlineUserCircle,
  HiOutlineUserGroup,
  HiOutlineUserPlus,
  HiOutlineUserMinus,
  HiOutlineUsers,
  
  // Misc
  HiOutlineHome,
  HiOutlineStar,
  HiOutlineHeart,
  HiOutlineBookmark,
  HiOutlineFlag,
  HiOutlineTag,
  HiOutlineHashtag,
  HiOutlineAtSymbol,
  HiOutlineLink,
  HiOutlinePaperClip,
  HiOutlineTrash,
  HiOutlinePencil,
  HiOutlinePencilSquare,
  HiOutlineSquare2Stack,
  HiOutlineArrowsPointingOut,
  HiOutlineArrowsPointingIn,
  HiOutlineQueueList,
  HiOutlineRectangleStack,
  HiOutlineRectangleGroup,
  HiOutlineCube,
  HiOutlineCubeTransparent,
  HiOutlineGlobeAlt,
  HiOutlineMap,
  HiOutlineMapPin,
  HiOutlineBuildingOffice,
  HiOutlineBuildingOffice2,
  HiOutlineCalculator,
  HiOutlineCurrencyDollar,
  HiOutlineScale,
  HiOutlineHandThumbUp,
  HiOutlineHandThumbDown,
  HiOutlinePlayCircle,
  HiOutlinePauseCircle,
  HiOutlineStopCircle,
  HiOutlinePlay,
  HiOutlinePause,
  HiOutlineStop,
  HiOutlineForward,
  HiOutlineBackward,
  HiOutlineSun,
  HiOutlineMoon,
  HiOutlineAcademicCap,
} from "solid-icons/hi";

// Re-export all icons explicitly
export {
  // Navigation & Actions
  HiOutlineArrowLeft,
  HiOutlineArrowRight,
  HiOutlineArrowUp,
  HiOutlineArrowDown,
  HiOutlineArrowPath,
  HiOutlineArrowUturnLeft,
  HiOutlineArrowUturnRight,
  HiOutlineArrowTopRightOnSquare,
  HiOutlineArrowDownTray,
  HiOutlineArrowUpTray,
  HiOutlineArrowRightOnRectangle,
  
  // Files & Folders
  HiOutlineFolder,
  HiOutlineFolderOpen,
  HiOutlineFolderPlus,
  HiOutlineFolderMinus,
  HiOutlineFolderArrowDown,
  HiOutlineDocument,
  HiOutlineDocumentText,
  HiOutlineDocumentDuplicate,
  HiOutlineDocumentArrowDown,
  HiOutlineDocumentArrowUp,
  HiOutlineDocumentMagnifyingGlass,
  HiOutlineDocumentCheck,
  HiOutlineDocumentPlus,
  HiOutlineDocumentMinus,
  HiOutlineArchiveBox,
  HiOutlineArchiveBoxArrowDown,
  HiOutlineClipboard,
  HiOutlineClipboardDocument,
  HiOutlineClipboardDocumentList,
  HiOutlineClipboardDocumentCheck,
  HiOutlineInbox,
  
  // UI Controls
  HiOutlineChevronDown,
  HiOutlineChevronUp,
  HiOutlineChevronLeft,
  HiOutlineChevronRight,
  HiOutlineChevronDoubleDown,
  HiOutlineChevronDoubleUp,
  HiOutlineChevronDoubleLeft,
  HiOutlineChevronDoubleRight,
  HiOutlineBars3,
  HiOutlineBars4,
  HiOutlineBars3BottomLeft,
  HiOutlineBars3BottomRight,
  HiOutlineEllipsisHorizontal,
  HiOutlineEllipsisVertical,
  HiOutlineXMark,
  HiOutlinePlus,
  HiOutlineMinus,
  HiOutlineCheck,
  HiOutlineCheckCircle,
  HiOutlineCheckBadge,
  HiOutlineXCircle,
  HiOutlinePlusCircle,
  HiOutlineMinusCircle,
  
  // Search & View
  HiOutlineMagnifyingGlass,
  HiOutlineMagnifyingGlassPlus,
  HiOutlineMagnifyingGlassMinus,
  HiOutlineMagnifyingGlassCircle,
  HiOutlineEye,
  HiOutlineEyeSlash,
  HiOutlineFunnel,
  HiOutlineAdjustmentsHorizontal,
  HiOutlineAdjustmentsVertical,
  HiOutlineViewColumns,
  HiOutlineSquares2x2,
  HiOutlineListBullet,
  HiOutlineTableCells,
  HiOutlineChartBar,
  HiOutlineChartPie,
  HiOutlineChartBarSquare,
  HiOutlineArrowsRightLeft,
  
  // Status & Alerts
  HiOutlineExclamationTriangle,
  HiOutlineExclamationCircle,
  HiOutlineInformationCircle,
  HiOutlineQuestionMarkCircle,
  HiOutlineBellAlert,
  HiOutlineBell,
  HiOutlineShieldCheck,
  HiOutlineShieldExclamation,
  
  // Data & Database
  HiOutlineCircleStack,
  HiOutlineServerStack,
  HiOutlineServer,
  HiOutlineCpuChip,
  HiOutlineCommandLine,
  HiOutlineCodeBracket,
  HiOutlineCodeBracketSquare,
  HiOutlineFingerPrint,
  HiOutlineKey,
  HiOutlineLockClosed,
  HiOutlineLockOpen,
  
  // Media & Content
  HiOutlinePhoto,
  HiOutlineFilm,
  HiOutlineMusicalNote,
  HiOutlineVideoCamera,
  HiOutlineCamera,
  HiOutlineMicrophone,
  HiOutlineSpeakerWave,
  HiOutlineSpeakerXMark,
  
  // Communication
  HiOutlineEnvelope,
  HiOutlineEnvelopeOpen,
  HiOutlineChatBubbleLeft,
  HiOutlineChatBubbleLeftRight,
  HiOutlineChatBubbleBottomCenter,
  HiOutlineChatBubbleOvalLeft,
  HiOutlinePhone,
  
  // Time & Calendar
  HiOutlineClock,
  HiOutlineCalendar,
  HiOutlineCalendarDays,
  
  // Settings & Tools
  HiOutlineCog6Tooth,
  HiOutlineCog8Tooth,
  HiOutlineWrench,
  HiOutlineWrenchScrewdriver,
  HiOutlinePaintBrush,
  HiOutlineSparkles,
  HiOutlineBolt,
  HiOutlineFire,
  HiOutlineBeaker,
  
  // Devices
  HiOutlineDevicePhoneMobile,
  HiOutlineDeviceTablet,
  HiOutlineComputerDesktop,
  HiOutlineTv,
  HiOutlinePrinter,
  HiOutlineWifi,
  HiOutlineSignal,
  
  // Storage
  HiOutlineCloudArrowDown,
  HiOutlineCloudArrowUp,
  HiOutlineCloud,
  
  // People & Users
  HiOutlineUser,
  HiOutlineUserCircle,
  HiOutlineUserGroup,
  HiOutlineUserPlus,
  HiOutlineUserMinus,
  HiOutlineUsers,
  
  // Misc
  HiOutlineHome,
  HiOutlineStar,
  HiOutlineHeart,
  HiOutlineBookmark,
  HiOutlineFlag,
  HiOutlineTag,
  HiOutlineHashtag,
  HiOutlineAtSymbol,
  HiOutlineLink,
  HiOutlinePaperClip,
  HiOutlineTrash,
  HiOutlinePencil,
  HiOutlinePencilSquare,
  HiOutlineSquare2Stack,
  HiOutlineArrowsPointingOut,
  HiOutlineArrowsPointingIn,
  HiOutlineQueueList,
  HiOutlineRectangleStack,
  HiOutlineRectangleGroup,
  HiOutlineCube,
  HiOutlineCubeTransparent,
  HiOutlineGlobeAlt,
  HiOutlineMap,
  HiOutlineMapPin,
  HiOutlineBuildingOffice,
  HiOutlineBuildingOffice2,
  HiOutlineCalculator,
  HiOutlineCurrencyDollar,
  HiOutlineScale,
  HiOutlineHandThumbUp,
  HiOutlineHandThumbDown,
  HiOutlinePlayCircle,
  HiOutlinePauseCircle,
  HiOutlineStopCircle,
  HiOutlinePlay,
  HiOutlinePause,
  HiOutlineStop,
  HiOutlineForward,
  HiOutlineBackward,
  HiOutlineSun,
  HiOutlineMoon,
  HiOutlineAcademicCap,
};

// ============================================================================
// Icon Props Type
// ============================================================================

export interface IconProps extends JSX.SvgSVGAttributes<SVGSVGElement> {
  /** Size preset or custom size */
  size?: "xs" | "sm" | "md" | "lg" | "xl" | number;
  /** Additional CSS classes */
  class?: string;
}

// Size mappings
const sizeMap: Record<string, string> = {
  xs: "w-3 h-3",
  sm: "w-4 h-4",
  md: "w-5 h-5",
  lg: "w-6 h-6",
  xl: "w-8 h-8",
};

// ============================================================================
// Icon Wrapper - Standardizes icon sizing
// ============================================================================

/**
 * Wrapper to apply consistent sizing to any icon
 */
export function withIconSize<P extends IconProps>(
  IconComponent: Component<P>
): Component<P> {
  return (props: P) => {
    const [local, rest] = splitProps(props as IconProps, ["size", "class"]);
    
    const sizeClass = () => {
      if (typeof local.size === "number") {
        return `w-[${local.size}px] h-[${local.size}px]`;
      }
      return sizeMap[local.size || "md"];
    };
    
    return (
      <IconComponent
        {...(rest as unknown as P)}
        class={`${sizeClass()} ${local.class || ""} shrink-0`}
      />
    );
  };
}

// ============================================================================
// Semantic Icon Mappings for Application Use
// ============================================================================

import {
  HiOutlineFolder as FolderIconBase,
  HiOutlineDocument as FileIconBase,
  HiOutlineDocumentText as FileTextIconBase,
  HiOutlineArchiveBox as ArchiveIconBase,
  HiOutlineCircleStack as DatabaseIconBase,
  HiOutlineDevicePhoneMobile as PhoneIconBase,
  HiOutlineCpuChip as ChipIconBase,
  HiOutlinePhoto as ImageIconBase,
  HiOutlineFilm as VideoIconBase,
  HiOutlineMusicalNote as AudioIconBase,
  HiOutlineCodeBracket as CodeIconBase,
  HiOutlineMagnifyingGlass as SearchIconBase,
  HiOutlineXMark as CloseIconBase,
  HiOutlineExclamationTriangle as WarningIconBase,
  HiOutlineChevronDown as ChevronDownIconBase,
  HiOutlineChevronRight as ChevronRightIconBase,
  HiOutlineClipboardDocument as CopyIconBase,
  HiOutlineClock as TimeIconBase,
  HiOutlineMapPin as LocationIconBase,
  HiOutlineEnvelope as EmailIconBase,
  HiOutlineSquares2x2 as ViewGridIconBase,
  HiOutlineListBullet as ViewListIconBase,
  HiOutlineArrowsPointingOut as ExpandIconBase,
} from "solid-icons/hi";

// ============================================================================
// Semantic Icon Components with Default Sizing
// Only includes wrappers that are actively used in the codebase.
// For other icons, import HiOutline* directly from this file.
// ============================================================================

// Files & Folders (used by getFileIcon)
export const FolderIcon: Component<IconProps> = (props) => <FolderIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const FileIcon: Component<IconProps> = (props) => <FileIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const FileTextIcon: Component<IconProps> = (props) => <FileTextIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ArchiveIcon: Component<IconProps> = (props) => <ArchiveIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const DatabaseIcon: Component<IconProps> = (props) => <DatabaseIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ChipIcon: Component<IconProps> = (props) => <ChipIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Media (used by getFileIcon)
export const ImageIcon: Component<IconProps> = (props) => <ImageIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const VideoIcon: Component<IconProps> = (props) => <VideoIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const AudioIcon: Component<IconProps> = (props) => <AudioIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const CodeIcon: Component<IconProps> = (props) => <CodeIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Actions (directly imported)
export const SearchIcon: Component<IconProps> = (props) => <SearchIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const CopyIcon: Component<IconProps> = (props) => <CopyIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// View (directly imported)
export const ExpandIcon: Component<IconProps> = (props) => <ExpandIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const HiOutlineViewGrid: Component<IconProps> = (props) => <ViewGridIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const HiOutlineViewList: Component<IconProps> = (props) => <ViewListIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const HiOutlineX: Component<IconProps> = (props) => <CloseIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Status (directly imported)
export const WarningIcon: Component<IconProps> = (props) => <WarningIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Chevrons (directly imported)
export const ChevronDownIcon: Component<IconProps> = (props) => <ChevronDownIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ChevronRightIcon: Component<IconProps> = (props) => <ChevronRightIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Time & Location (directly imported)
export const TimeIcon: Component<IconProps> = (props) => <TimeIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const LocationIcon: Component<IconProps> = (props) => <LocationIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Communication (directly imported)
export const EmailIcon: Component<IconProps> = (props) => <EmailIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// ============================================================================
// Forensic-Specific Icon Components
// ============================================================================

// Container type icons - colors match ui/constants CONTAINER_ICON_COLORS
export const AD1Icon: Component<IconProps> = (props) => <ArchiveIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0 text-blue-400`} />;
export const E01Icon: Component<IconProps> = (props) => <DatabaseIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0 text-green-400`} />;
export const L01Icon: Component<IconProps> = (props) => <FileTextIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0 text-yellow-400`} />;
export const RawIcon: Component<IconProps> = (props) => <ChipIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0 text-purple-400`} />;
export const UFEDIcon: Component<IconProps> = (props) => <PhoneIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0 text-accent`} />;
export const ZipIcon: Component<IconProps> = (props) => <ArchiveIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0 text-orange-400`} />;

// Database type icons for processed databases
export const AxiomIcon: Component<IconProps> = (props) => <DatabaseIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0 text-accent`} />;
export const CellebriteIcon: Component<IconProps> = (props) => <PhoneIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0 text-pink-400`} />;
export const GenericDbIcon: Component<IconProps> = (props) => <FolderIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0 text-txt-secondary`} />;

// ============================================================================
// Icon Getter Functions (for dynamic icon selection)
// ============================================================================

/**
 * Get container type icon component based on type string
 */
export function getContainerTypeIcon(type: string): Component<IconProps> {
  const t = type.toLowerCase();
  if (t.includes("ad1")) return AD1Icon;
  if (t.includes("e01") || t.includes("encase")) return E01Icon;
  if (t.includes("l01")) return L01Icon;
  if (t.includes("raw") || t.includes("dd")) return RawIcon;
  if (t.includes("ufed") || t.includes("ufd")) return UFEDIcon;
  if (t.includes("tar") || t.includes("7z") || t.includes("zip") || t.includes("rar") || t.includes("gz")) return ZipIcon;
  return FileIcon;
}

/**
 * Get database type icon component
 */
export function getDatabaseTypeIcon(dbType: string): Component<IconProps> {
  const t = dbType.toLowerCase();
  if (t.includes("axiom") || t.includes("magnet")) return AxiomIcon;
  if (t.includes("cellebrite") || t.includes("pa")) return CellebriteIcon;
  return GenericDbIcon;
}

/**
 * Get file icon based on extension
 */
export function getFileIcon(filename: string, isDir: boolean): Component<IconProps> {
  if (isDir) return FolderIcon;
  
  const ext = getExtension(filename);
  
  // Images
  if (['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg', 'webp', 'ico', 'tiff'].includes(ext)) {
    return ImageIcon;
  }
  
  // Videos
  if (['mp4', 'avi', 'mkv', 'mov', 'wmv', 'flv', 'webm', 'm4v'].includes(ext)) {
    return VideoIcon;
  }
  
  // Audio
  if (['mp3', 'wav', 'ogg', 'flac', 'aac', 'm4a', 'wma'].includes(ext)) {
    return AudioIcon;
  }
  
  // Code
  if (['js', 'ts', 'jsx', 'tsx', 'py', 'rb', 'java', 'c', 'cpp', 'h', 'cs', 'go', 'rs', 'php', 'html', 'css', 'json', 'xml', 'yaml', 'yml', 'sql'].includes(ext)) {
    return CodeIcon;
  }
  
  // Text/Documents
  if (['txt', 'md', 'rtf', 'doc', 'docx', 'pdf', 'odt'].includes(ext)) {
    return FileTextIcon;
  }
  
  // Archives
  if (['zip', 'tar', 'gz', 'rar', '7z', 'bz2', 'xz'].includes(ext)) {
    return ArchiveIcon;
  }
  
  // Database
  if (['db', 'sqlite', 'sqlite3', 'mdb', 'accdb'].includes(ext)) {
    return DatabaseIcon;
  }
  
  return FileIcon;
}

// ============================================================================
// Loading/Spinner Icon
// ============================================================================

export const SpinnerIcon: Component<IconProps> = (props) => (
  <svg 
    class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0 animate-spin`}
    xmlns="http://www.w3.org/2000/svg" 
    fill="none" 
    viewBox="0 0 24 24"
  >
    <circle 
      class="opacity-25" 
      cx="12" 
      cy="12" 
      r="10" 
      stroke="currentColor" 
      stroke-width="4"
    />
    <path 
      class="opacity-75" 
      fill="currentColor" 
      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
    />
  </svg>
);
