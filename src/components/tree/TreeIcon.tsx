// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TreeIcon - File/folder icons with consistent styling
 * 
 * Maps file extensions and types to appropriate icons with color coding.
 */

import type { JSX } from 'solid-js';
import {
  HiOutlineFolder,
  HiOutlineFolderOpen,
  HiOutlineDocument,
  HiOutlineDocumentText,
  HiOutlinePhoto,
  HiOutlineTableCells,
  HiOutlineArchiveBox,
  HiOutlineCog6Tooth,
  HiOutlineCircleStack,
  HiOutlineMusicalNote,
  HiOutlineFilm,
  HiOutlineCodeBracket,
  HiOutlineGlobeAlt,
  HiOutlineLockClosed,
  HiOutlineEnvelope,
  HiOutlineDevicePhoneMobile,
} from '../icons';
import { getExtension } from '../../utils';

export interface TreeIconProps {
  /** File/folder name (used to determine extension) */
  name: string;
  /** Whether this is a directory */
  isDir: boolean;
  /** Whether the directory is expanded (for folder icon state) */
  isExpanded?: boolean;
  /** Custom class names */
  class?: string;
  /** Entry type hint (for UFED, etc.) */
  entryType?: string;
}

/** File extension to icon mapping */
const EXTENSION_ICONS: Record<string, (cls: string) => JSX.Element> = {
  // Images
  jpg: (cls) => <HiOutlinePhoto class={`${cls} text-pink-400`} />,
  jpeg: (cls) => <HiOutlinePhoto class={`${cls} text-pink-400`} />,
  png: (cls) => <HiOutlinePhoto class={`${cls} text-pink-400`} />,
  gif: (cls) => <HiOutlinePhoto class={`${cls} text-pink-400`} />,
  bmp: (cls) => <HiOutlinePhoto class={`${cls} text-pink-400`} />,
  ico: (cls) => <HiOutlinePhoto class={`${cls} text-pink-400`} />,
  webp: (cls) => <HiOutlinePhoto class={`${cls} text-pink-400`} />,
  svg: (cls) => <HiOutlinePhoto class={`${cls} text-pink-400`} />,
  
  // Documents
  doc: (cls) => <HiOutlineDocumentText class={`${cls} text-blue-400`} />,
  docx: (cls) => <HiOutlineDocumentText class={`${cls} text-blue-400`} />,
  pdf: (cls) => <HiOutlineDocumentText class={`${cls} text-red-400`} />,
  txt: (cls) => <HiOutlineDocumentText class={`${cls} text-txt-secondary`} />,
  rtf: (cls) => <HiOutlineDocumentText class={`${cls} text-blue-400`} />,
  md: (cls) => <HiOutlineDocumentText class={`${cls} text-txt-secondary`} />,
  
  // Spreadsheets
  xls: (cls) => <HiOutlineTableCells class={`${cls} text-green-400`} />,
  xlsx: (cls) => <HiOutlineTableCells class={`${cls} text-green-400`} />,
  csv: (cls) => <HiOutlineTableCells class={`${cls} text-green-400`} />,
  
  // Archives - Expanded with all formats
  zip: (cls) => <HiOutlineArchiveBox class={`${cls} text-yellow-400`} />,
  rar: (cls) => <HiOutlineArchiveBox class={`${cls} text-purple-400`} />,
  '7z': (cls) => <HiOutlineArchiveBox class={`${cls} text-accent`} />,
  tar: (cls) => <HiOutlineArchiveBox class={`${cls} text-orange-400`} />,
  gz: (cls) => <HiOutlineArchiveBox class={`${cls} text-orange-400`} />,
  tgz: (cls) => <HiOutlineArchiveBox class={`${cls} text-orange-400`} />,
  bz2: (cls) => <HiOutlineArchiveBox class={`${cls} text-amber-400`} />,
  xz: (cls) => <HiOutlineArchiveBox class={`${cls} text-blue-400`} />,
  txz: (cls) => <HiOutlineArchiveBox class={`${cls} text-blue-400`} />,
  zst: (cls) => <HiOutlineArchiveBox class={`${cls} text-green-400`} />,
  lz4: (cls) => <HiOutlineArchiveBox class={`${cls} text-lime-400`} />,
  r00: (cls) => <HiOutlineArchiveBox class={`${cls} text-purple-400`} />,
  r01: (cls) => <HiOutlineArchiveBox class={`${cls} text-purple-400`} />,
  
  // Executables/System
  exe: (cls) => <HiOutlineCog6Tooth class={`${cls} text-txt-secondary`} />,
  dll: (cls) => <HiOutlineCog6Tooth class={`${cls} text-txt-muted`} />,
  sys: (cls) => <HiOutlineCog6Tooth class={`${cls} text-txt-muted`} />,
  
  // Databases
  db: (cls) => <HiOutlineCircleStack class={`${cls} text-purple-400`} />,
  sqlite: (cls) => <HiOutlineCircleStack class={`${cls} text-purple-400`} />,
  sqlite3: (cls) => <HiOutlineCircleStack class={`${cls} text-purple-400`} />,
  mdb: (cls) => <HiOutlineCircleStack class={`${cls} text-purple-400`} />,
  
  // Audio
  mp3: (cls) => <HiOutlineMusicalNote class={`${cls} text-accent`} />,
  wav: (cls) => <HiOutlineMusicalNote class={`${cls} text-accent`} />,
  flac: (cls) => <HiOutlineMusicalNote class={`${cls} text-accent`} />,
  m4a: (cls) => <HiOutlineMusicalNote class={`${cls} text-accent`} />,
  
  // Video
  mp4: (cls) => <HiOutlineFilm class={`${cls} text-purple-400`} />,
  avi: (cls) => <HiOutlineFilm class={`${cls} text-purple-400`} />,
  mkv: (cls) => <HiOutlineFilm class={`${cls} text-purple-400`} />,
  mov: (cls) => <HiOutlineFilm class={`${cls} text-purple-400`} />,
  
  // Code
  js: (cls) => <HiOutlineCodeBracket class={`${cls} text-yellow-400`} />,
  ts: (cls) => <HiOutlineCodeBracket class={`${cls} text-blue-400`} />,
  py: (cls) => <HiOutlineCodeBracket class={`${cls} text-green-400`} />,
  rs: (cls) => <HiOutlineCodeBracket class={`${cls} text-orange-400`} />,
  json: (cls) => <HiOutlineCodeBracket class={`${cls} text-txt-secondary`} />,
  xml: (cls) => <HiOutlineCodeBracket class={`${cls} text-orange-400`} />,
  html: (cls) => <HiOutlineGlobeAlt class={`${cls} text-orange-400`} />,
  css: (cls) => <HiOutlineCodeBracket class={`${cls} text-blue-400`} />,
  
  // Encrypted/Keys
  pem: (cls) => <HiOutlineLockClosed class={`${cls} text-red-400`} />,
  key: (cls) => <HiOutlineLockClosed class={`${cls} text-red-400`} />,
  
  // Email
  eml: (cls) => <HiOutlineEnvelope class={`${cls} text-blue-400`} />,
  msg: (cls) => <HiOutlineEnvelope class={`${cls} text-blue-400`} />,
  
  // Log files
  log: (cls) => <HiOutlineDocumentText class={`${cls} text-txt-muted`} />,
};


export function TreeIcon(props: TreeIconProps) {
  const iconClass = () => `w-4 h-4 shrink-0 ${props.class || ''}`;
  
  // Directory icons
  if (props.isDir) {
    return props.isExpanded 
      ? <HiOutlineFolderOpen class={`${iconClass()} text-amber-400`} />
      : <HiOutlineFolder class={`${iconClass()} text-amber-400`} />;
  }
  
  // UFED-specific entry types
  if (props.entryType) {
    const type = props.entryType.toLowerCase();
    if (type.includes('phone') || type.includes('device')) {
      return <HiOutlineDevicePhoneMobile class={`${iconClass()} text-accent`} />;
    }
  }
  
  // Get extension from filename
  const ext = getExtension(props.name);
  
  // Check for matching icon
  const iconFn = EXTENSION_ICONS[ext];
  if (iconFn) {
    return iconFn(iconClass());
  }
  
  // Default document icon
  return <HiOutlineDocument class={`${iconClass()} text-txt-secondary`} />;
}
