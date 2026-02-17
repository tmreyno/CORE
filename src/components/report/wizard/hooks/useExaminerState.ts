// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, type Accessor, type Setter } from "solid-js";
import type { ExaminerInfo } from "../../types";
import { getPreference } from "../../../preferences";

export interface ExaminerState {
  examiner: Accessor<ExaminerInfo>;
  setExaminer: Setter<ExaminerInfo>;
  newCert: Accessor<string>;
  setNewCert: Setter<string>;
  addCertification: () => void;
  removeCertification: (cert: string) => void;
}

export function useExaminerState(): ExaminerState {
  const savedCerts = getPreference("examinerCertifications");

  const [examiner, setExaminer] = createSignal<ExaminerInfo>({
    name: getPreference("examinerName") || "",
    title: getPreference("examinerTitle") || undefined,
    organization: getPreference("organizationName") || undefined,
    email: getPreference("examinerEmail") || undefined,
    phone: getPreference("examinerPhone") || undefined,
    badge_number: getPreference("examinerBadge") || undefined,
    certifications: Array.isArray(savedCerts) && savedCerts.length > 0 ? savedCerts : [],
  });

  const [newCert, setNewCert] = createSignal("");

  const addCertification = () => {
    const cert = newCert().trim();
    if (cert && !examiner().certifications.includes(cert)) {
      setExaminer({
        ...examiner(),
        certifications: [...examiner().certifications, cert],
      });
      setNewCert("");
    }
  };

  const removeCertification = (cert: string) => {
    setExaminer({
      ...examiner(),
      certifications: examiner().certifications.filter((c) => c !== cert),
    });
  };

  return {
    examiner,
    setExaminer,
    newCert,
    setNewCert,
    addCertification,
    removeCertification,
  };
}
