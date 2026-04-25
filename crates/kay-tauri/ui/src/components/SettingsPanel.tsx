// Tauri UI — Settings Panel Component
// Phase 10 Wave 7: Settings panel

import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ProjectSettings {
  model_id: string;
  temperature: number;
  max_tokens: number;
  approval_mode: "off" | "first_use" | "always";
  sandbox_enabled: boolean;
  verifier_enabled: boolean;
}

interface SettingsPanelProps {
  projectPath: string;
  onClose: () => void;
}

type TabId = "session" | "model" | "verifier" | "sandbox";

const tabs: { id: TabId; label: string }[] = [
  { id: "session", label: "Session" },
  { id: "model", label: "Model" },
  { id: "verifier", label: "Verifier" },
  { id: "sandbox", label: "Sandbox" },
];

const approvalModes: { value: ProjectSettings["approval_mode"]; label: string }[] = [
  { value: "off", label: "Off (no approval)" },
  { value: "first_use", label: "First use per tool" },
  { value: "always", label: "Always approve" },
];

export function SettingsPanel({ projectPath, onClose }: SettingsPanelProps) {
  const [activeTab, setActiveTab] = useState<TabId>("session");
  const [settings, setSettings] = useState<ProjectSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load settings on mount
  useEffect(() => {
    loadSettings();
  }, [projectPath]);

  async function loadSettings() {
    setLoading(true);
    setError(null);
    try {
      const loaded = await invoke<ProjectSettings | null>("load_project_settings", {
        projectPath,
      });
      // Use defaults if no settings exist
      setSettings(
        loaded ?? {
          model_id: "openai/gpt-4o",
          temperature: 0.7,
          max_tokens: 4096,
          approval_mode: "first_use",
          sandbox_enabled: true,
          verifier_enabled: true,
        }
      );
    } catch (e) {
      setError(`Failed to load settings: ${e}`);
    } finally {
      setLoading(false);
    }
  }

  async function handleSave() {
    if (!settings) return;
    setSaving(true);
    setError(null);
    try {
      await invoke("save_project_settings", {
        settings,
        projectPath,
      });
      onClose();
    } catch (e) {
      setError(`Failed to save settings: ${e}`);
    } finally {
      setSaving(false);
    }
  }

  function updateSetting<K extends keyof ProjectSettings>(
    key: K,
    value: ProjectSettings[K]
  ) {
    setSettings((prev) => (prev ? { ...prev, [key]: value } : prev));
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  if (loading) {
    return (
      <div className="settings-panel" onKeyDown={handleKeyDown}>
        <div className="settings-content">
          <p className="settings-loading">Loading settings...</p>
        </div>
      </div>
    );
  }

  if (!settings) return null;

  return (
    <div className="settings-panel" onKeyDown={handleKeyDown}>
      <div className="settings-header">
        <h2>Settings</h2>
        <button className="settings-close" onClick={onClose} aria-label="Close">
          ✕
        </button>
      </div>

      <div className="settings-tabs">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`settings-tab ${activeTab === tab.id ? "active" : ""}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </div>

      <div className="settings-content">
        {error && <div className="settings-error">{error}</div>}

        {activeTab === "session" && (
          <div className="settings-section">
            <h3>Session Settings</h3>
            <p className="settings-description">
              Control how Kay manages your sessions.
            </p>

            <label className="settings-field">
              <span>Command Approval</span>
              <select
                value={settings.approval_mode}
                onChange={(e) =>
                  updateSetting(
                    "approval_mode",
                    e.target.value as ProjectSettings["approval_mode"]
                  )
                }
              >
                {approvalModes.map((mode) => (
                  <option key={mode.value} value={mode.value}>
                    {mode.label}
                  </option>
                ))}
              </select>
            </label>
          </div>
        )}

        {activeTab === "model" && (
          <div className="settings-section">
            <h3>Model Settings</h3>
            <p className="settings-description">
              Configure the AI model for this project.
            </p>

            <label className="settings-field">
              <span>Model ID</span>
              <input
                type="text"
                value={settings.model_id}
                onChange={(e) => updateSetting("model_id", e.target.value)}
                placeholder="openai/gpt-4o"
              />
            </label>

            <label className="settings-field">
              <span>Temperature</span>
              <input
                type="range"
                min="0"
                max="2"
                step="0.1"
                value={settings.temperature}
                onChange={(e) =>
                  updateSetting("temperature", parseFloat(e.target.value))
                }
              />
              <span className="settings-value">{settings.temperature}</span>
            </label>

            <label className="settings-field">
              <span>Max Tokens</span>
              <input
                type="number"
                value={settings.max_tokens}
                onChange={(e) =>
                  updateSetting("max_tokens", parseInt(e.target.value, 10))
                }
                min="256"
                max="32768"
              />
            </label>
          </div>
        )}

        {activeTab === "verifier" && (
          <div className="settings-section">
            <h3>Verifier Settings</h3>
            <p className="settings-description">
              Control code verification behavior.
            </p>

            <label className="settings-field">
              <span>Enable Verifier</span>
              <input
                type="checkbox"
                checked={settings.verifier_enabled}
                onChange={(e) => updateSetting("verifier_enabled", e.target.checked)}
              />
            </label>
          </div>
        )}

        {activeTab === "sandbox" && (
          <div className="settings-section">
            <h3>Sandbox Settings</h3>
            <p className="settings-description">
              Configure sandbox isolation for tool execution.
            </p>

            <label className="settings-field">
              <span>Enable Sandbox</span>
              <input
                type="checkbox"
                checked={settings.sandbox_enabled}
                onChange={(e) => updateSetting("sandbox_enabled", e.target.checked)}
              />
            </label>
          </div>
        )}
      </div>

      <div className="settings-footer">
        <button className="settings-button secondary" onClick={onClose}>
          Cancel
        </button>
        <button
          className="settings-button primary"
          onClick={handleSave}
          disabled={saving}
        >
          {saving ? "Saving..." : "Save"}
        </button>
      </div>
    </div>
  );
}
