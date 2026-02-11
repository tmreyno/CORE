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
  HiOutlineFolderOpen as FolderOpenIconBase,
  HiOutlineDocument as FileIconBase,
  HiOutlineDocumentText as FileTextIconBase,
  HiOutlineArchiveBox as ArchiveIconBase,
  HiOutlineCircleStack as DatabaseIconBase,
  HiOutlineDevicePhoneMobile as PhoneIconBase,
  HiOutlineComputerDesktop as DesktopIconBase,
  HiOutlineCpuChip as ChipIconBase,
  HiOutlinePhoto as ImageIconBase,
  HiOutlineFilm as VideoIconBase,
  HiOutlineMusicalNote as AudioIconBase,
  HiOutlineCodeBracket as CodeIconBase,
  HiOutlineCog6Tooth as SettingsIconBase,
  HiOutlineMagnifyingGlass as SearchIconBase,
  HiOutlineAdjustmentsHorizontal as FilterIconBase,
  HiOutlineArrowPath as RefreshIconBase,
  HiOutlineXMark as CloseIconBase,
  HiOutlinePlus as PlusIconBase,
  HiOutlineMinus as MinusIconBase,
  HiOutlineCheck as CheckIconBase,
  HiOutlineCheckCircle as SuccessIconBase,
  HiOutlineXCircle as ErrorIconBase,
  HiOutlineExclamationTriangle as WarningIconBase,
  HiOutlineInformationCircle as InfoIconBase,
  HiOutlineQuestionMarkCircle as HelpIconBase,
  HiOutlineChevronDown as ChevronDownIconBase,
  HiOutlineChevronUp as ChevronUpIconBase,
  HiOutlineChevronLeft as ChevronLeftIconBase,
  HiOutlineChevronRight as ChevronRightIconBase,
  HiOutlineHome as HomeIconBase,
  HiOutlineTrash as TrashIconBase,
  HiOutlinePencil as EditIconBase,
  HiOutlineClipboardDocument as CopyIconBase,
  HiOutlineArrowDownTray as DownloadIconBase,
  HiOutlineArrowUpTray as UploadIconBase,
  HiOutlineEye as ViewIconBase,
  HiOutlineEyeSlash as HideIconBase,
  HiOutlineSun as LightModeIconBase,
  HiOutlineMoon as DarkModeIconBase,
  HiOutlineComputerDesktop as SystemModeIconBase,
  HiOutlineFingerPrint as HashIconBase,
  HiOutlineShieldCheck as VerifiedIconBase,
  HiOutlineShieldExclamation as UnverifiedIconBase,
  HiOutlineLockClosed as LockedIconBase,
  HiOutlineLockOpen as UnlockedIconBase,
  HiOutlineArrowUturnLeft as UndoIconBase,
  HiOutlineArrowUturnRight as RedoIconBase,
  HiOutlineBars3 as MenuIconBase,
  HiOutlineEllipsisVertical as MoreIconBase,
  HiOutlineTag as TagIconBase,
  HiOutlineBookmark as BookmarkIconBase,
  HiOutlineStar as StarIconBase,
  HiOutlineFlag as FlagIconBase,
  HiOutlineLink as LinkIconBase,
  HiOutlinePlayCircle as PlayIconBase,
  HiOutlinePauseCircle as PauseIconBase,
  HiOutlineStopCircle as StopIconBase,
  HiOutlineClock as TimeIconBase,
  HiOutlineCalendar as CalendarIconBase,
  HiOutlineGlobeAlt as GlobeIconBase,
  HiOutlineMapPin as LocationIconBase,
  HiOutlineUser as UserIconBase,
  HiOutlineUserGroup as UsersIconBase,
  HiOutlineEnvelope as EmailIconBase,
  HiOutlineChatBubbleLeft as ChatIconBase,
  HiOutlineCommandLine as TerminalIconBase,
  HiOutlineRectangleStack as LayersIconBase,
  HiOutlineSquares2x2 as GridIconBase,
  HiOutlineListBullet as ListIconBase,
  HiOutlineTableCells as TableIconBase,
  HiOutlineViewColumns as ColumnsIconBase,
  HiOutlineSquares2x2 as ViewGridIconBase,
  HiOutlineListBullet as ViewListIconBase,
  HiOutlineArrowsPointingOut as ExpandIconBase,
  HiOutlineArrowsPointingIn as CollapseIconBase,
  HiOutlineDocumentDuplicate as DuplicateIconBase,
  HiOutlineArrowTopRightOnSquare as ExternalLinkIconBase,
  HiOutlineFolderPlus as NewFolderIconBase,
  HiOutlineDocumentPlus as NewFileIconBase,
  HiOutlineCloudArrowDown as CloudDownloadIconBase,
  HiOutlineCloudArrowUp as CloudUploadIconBase,
  HiOutlineServer as ServerIconBase,
  HiOutlineWifi as NetworkIconBase,
  HiOutlineBolt as BoltIconBase,
  HiOutlineSparkles as SparklesIconBase,
  HiOutlineBeaker as BeakerIconBase,
  HiOutlinePaintBrush as ThemeIconBase,
  HiOutlineCalculator as CalculatorIconBase,
  HiOutlineScale as ScaleIconBase,
  HiOutlineCubeTransparent as ContainerIconBase,
  HiOutlineCube as PackageIconBase,
} from "solid-icons/hi";

// ============================================================================
// Semantic Icon Components with Default Sizing
// ============================================================================

// Navigation
export const FolderIcon: Component<IconProps> = (props) => <FolderIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const FolderOpenIcon: Component<IconProps> = (props) => <FolderOpenIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const FileIcon: Component<IconProps> = (props) => <FileIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const FileTextIcon: Component<IconProps> = (props) => <FileTextIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ArchiveIcon: Component<IconProps> = (props) => <ArchiveIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const DatabaseIcon: Component<IconProps> = (props) => <DatabaseIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const PhoneIcon: Component<IconProps> = (props) => <PhoneIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const DesktopIcon: Component<IconProps> = (props) => <DesktopIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ChipIcon: Component<IconProps> = (props) => <ChipIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Media
export const ImageIcon: Component<IconProps> = (props) => <ImageIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const VideoIcon: Component<IconProps> = (props) => <VideoIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const AudioIcon: Component<IconProps> = (props) => <AudioIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const CodeIcon: Component<IconProps> = (props) => <CodeIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Actions
export const SettingsIcon: Component<IconProps> = (props) => <SettingsIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const SearchIcon: Component<IconProps> = (props) => <SearchIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const FilterIcon: Component<IconProps> = (props) => <FilterIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const RefreshIcon: Component<IconProps> = (props) => <RefreshIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const CloseIcon: Component<IconProps> = (props) => <CloseIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const PlusIcon: Component<IconProps> = (props) => <PlusIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const MinusIcon: Component<IconProps> = (props) => <MinusIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const CheckIcon: Component<IconProps> = (props) => <CheckIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const TrashIcon: Component<IconProps> = (props) => <TrashIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const EditIcon: Component<IconProps> = (props) => <EditIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const CopyIcon: Component<IconProps> = (props) => <CopyIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const DownloadIcon: Component<IconProps> = (props) => <DownloadIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const UploadIcon: Component<IconProps> = (props) => <UploadIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const UndoIcon: Component<IconProps> = (props) => <UndoIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const RedoIcon: Component<IconProps> = (props) => <RedoIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// View
export const ViewIcon: Component<IconProps> = (props) => <ViewIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const HideIcon: Component<IconProps> = (props) => <HideIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ExpandIcon: Component<IconProps> = (props) => <ExpandIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const CollapseIcon: Component<IconProps> = (props) => <CollapseIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const GridIcon: Component<IconProps> = (props) => <GridIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ListIcon: Component<IconProps> = (props) => <ListIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const TableIcon: Component<IconProps> = (props) => <TableIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ColumnsIcon: Component<IconProps> = (props) => <ColumnsIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const LayersIcon: Component<IconProps> = (props) => <LayersIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const HiOutlineViewGrid: Component<IconProps> = (props) => <ViewGridIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const HiOutlineViewList: Component<IconProps> = (props) => <ViewListIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const HiOutlineX: Component<IconProps> = (props) => <CloseIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Status
export const SuccessIcon: Component<IconProps> = (props) => <SuccessIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ErrorIcon: Component<IconProps> = (props) => <ErrorIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const WarningIcon: Component<IconProps> = (props) => <WarningIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const InfoIcon: Component<IconProps> = (props) => <InfoIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const HelpIcon: Component<IconProps> = (props) => <HelpIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Chevrons
export const ChevronDownIcon: Component<IconProps> = (props) => <ChevronDownIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ChevronUpIcon: Component<IconProps> = (props) => <ChevronUpIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ChevronLeftIcon: Component<IconProps> = (props) => <ChevronLeftIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ChevronRightIcon: Component<IconProps> = (props) => <ChevronRightIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Theme
export const LightModeIcon: Component<IconProps> = (props) => <LightModeIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const DarkModeIcon: Component<IconProps> = (props) => <DarkModeIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const SystemModeIcon: Component<IconProps> = (props) => <SystemModeIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ThemeIcon: Component<IconProps> = (props) => <ThemeIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Security
export const HashIcon: Component<IconProps> = (props) => <HashIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const VerifiedIcon: Component<IconProps> = (props) => <VerifiedIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const UnverifiedIcon: Component<IconProps> = (props) => <UnverifiedIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const LockedIcon: Component<IconProps> = (props) => <LockedIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const UnlockedIcon: Component<IconProps> = (props) => <UnlockedIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Menu
export const MenuIcon: Component<IconProps> = (props) => <MenuIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const MoreIcon: Component<IconProps> = (props) => <MoreIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const HomeIcon: Component<IconProps> = (props) => <HomeIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Metadata
export const TagIcon: Component<IconProps> = (props) => <TagIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const BookmarkIcon: Component<IconProps> = (props) => <BookmarkIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const StarIcon: Component<IconProps> = (props) => <StarIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const FlagIcon: Component<IconProps> = (props) => <FlagIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const LinkIcon: Component<IconProps> = (props) => <LinkIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Playback
export const PlayIcon: Component<IconProps> = (props) => <PlayIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const PauseIcon: Component<IconProps> = (props) => <PauseIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const StopIcon: Component<IconProps> = (props) => <StopIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Time
export const TimeIcon: Component<IconProps> = (props) => <TimeIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const CalendarIcon: Component<IconProps> = (props) => <CalendarIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Location
export const GlobeIcon: Component<IconProps> = (props) => <GlobeIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const LocationIcon: Component<IconProps> = (props) => <LocationIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// People
export const UserIcon: Component<IconProps> = (props) => <UserIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const UsersIcon: Component<IconProps> = (props) => <UsersIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Communication
export const EmailIcon: Component<IconProps> = (props) => <EmailIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ChatIcon: Component<IconProps> = (props) => <ChatIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Technical
export const TerminalIcon: Component<IconProps> = (props) => <TerminalIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ServerIcon: Component<IconProps> = (props) => <ServerIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const NetworkIcon: Component<IconProps> = (props) => <NetworkIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Cloud
export const CloudDownloadIcon: Component<IconProps> = (props) => <CloudDownloadIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const CloudUploadIcon: Component<IconProps> = (props) => <CloudUploadIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// File Operations
export const DuplicateIcon: Component<IconProps> = (props) => <DuplicateIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ExternalLinkIcon: Component<IconProps> = (props) => <ExternalLinkIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const NewFolderIcon: Component<IconProps> = (props) => <NewFolderIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const NewFileIcon: Component<IconProps> = (props) => <NewFileIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

// Misc
export const BoltIcon: Component<IconProps> = (props) => <BoltIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const SparklesIcon: Component<IconProps> = (props) => <SparklesIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const BeakerIcon: Component<IconProps> = (props) => <BeakerIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const CalculatorIcon: Component<IconProps> = (props) => <CalculatorIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ScaleIcon: Component<IconProps> = (props) => <ScaleIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const ContainerIcon: Component<IconProps> = (props) => <ContainerIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;
export const PackageIcon: Component<IconProps> = (props) => <PackageIconBase class={`${sizeMap[props.size as string || "md"]} ${props.class || ""} shrink-0`} />;

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
