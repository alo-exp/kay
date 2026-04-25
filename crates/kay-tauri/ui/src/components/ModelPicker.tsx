// Tauri UI — Model Picker Component
// Phase 10 Wave 7: Model tier selection
// Success criteria: tiered list (Recommended/Experimental/All)

import React, { useState } from "react";

export type ModelTier = "Recommended" | "Experimental" | "All";

interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  context_window: number;
  input_cost_per_mtok: number;
  output_cost_per_mtok: number;
}

// Known models per tier from Phase 5 spec
const MODELS: Record<ModelTier, ModelInfo[]> = {
  Recommended: [
    {
      id: "anthropic/claude-4-sonnet",
      name: "Claude 4 Sonnet",
      provider: "Anthropic",
      context_window: 200000,
      input_cost_per_mtok: 3.0,
      output_cost_per_mtok: 15.0,
    },
    {
      id: "openai/gpt-4o",
      name: "GPT-4o",
      provider: "OpenAI",
      context_window: 128000,
      input_cost_per_mtok: 2.5,
      output_cost_per_mtok: 10.0,
    },
    {
      id: "google/gemini-2.5-flash",
      name: "Gemini 2.5 Flash",
      provider: "Google",
      context_window: 1000000,
      input_cost_per_mtok: 0.075,
      output_cost_per_mtok: 0.3,
    },
  ],
  Experimental: [
    {
      id: "anthropic/claude-4-opus",
      name: "Claude 4 Opus",
      provider: "Anthropic",
      context_window: 200000,
      input_cost_per_mtok: 15.0,
      output_cost_per_mtok: 75.0,
    },
    {
      id: "openai/o3",
      name: "OpenAI o3",
      provider: "OpenAI",
      context_window: 200000,
      input_cost_per_mtok: 10.0,
      output_cost_per_mtok: 40.0,
    },
    {
      id: "meta/llama-4",
      name: "Llama 4",
      provider: "Meta",
      context_window: 1000000,
      input_cost_per_mtok: 0.5,
      output_cost_per_mtok: 2.0,
    },
  ],
  All: [], // Any model not explicitly allowlisted
};

interface ModelPickerProps {
  selectedModel: string;
  selectedTier: ModelTier;
  onModelSelect: (modelId: string) => void;
  onTierChange: (tier: ModelTier) => void;
}

export function ModelPicker({
  selectedModel,
  selectedTier,
  onModelSelect,
  onTierChange,
}: ModelPickerProps) {
  const [showAllModels, setShowAllModels] = useState(false);
  const [customModelInput, setCustomModelInput] = useState("");

  function formatCost(cost: number): string {
    return `$${cost.toFixed(3)}/1K tokens`;
  }

  function formatContext(window: number): string {
    if (window >= 1000000) return `${window / 1000000}M ctx`;
    if (window >= 1000) return `${window / 1000}K ctx`;
    return `${window} ctx`;
  }

  function handleCustomModelSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (customModelInput.trim()) {
      onModelSelect(customModelInput.trim());
      setCustomModelInput("");
    }
  }

  const currentModels = MODELS[selectedTier];
  const isRecommended = selectedTier === "Recommended";

  return (
    <div className="model-picker">
      <div className="model-picker-header">
        <h3>Model Selection</h3>
      </div>

      {/* Tier selector */}
      <div className="model-tier-tabs">
        {(["Recommended", "Experimental", "All"] as ModelTier[]).map((tier) => (
          <button
            key={tier}
            className={`model-tier-tab ${selectedTier === tier ? "active" : ""} ${tier === "Recommended" ? "recommended" : ""}`}
            onClick={() => onTierChange(tier)}
          >
            {tier}
            {tier === "Recommended" && <span className="tier-badge">✓</span>}
          </button>
        ))}
      </div>

      {/* Warning for All tier */}
      {selectedTier === "All" && (
        <div className="model-picker-warning">
          ⚠️ Compatibility unknown for unlisted models. Use at your own risk.
        </div>
      )}

      {/* Model list */}
      <div className="model-list">
        {currentModels.map((model) => (
          <div
            key={model.id}
            className={`model-item ${selectedModel === model.id ? "selected" : ""}`}
            onClick={() => onModelSelect(model.id)}
          >
            <div className="model-item-header">
              <span className="model-name">{model.name}</span>
              {isRecommended && <span className="model-verified">✓</span>}
            </div>
            <div className="model-item-meta">
              <span className="model-provider">{model.provider}</span>
              <span className="model-context">{formatContext(model.context_window)}</span>
            </div>
            <div className="model-item-cost">
              <span>In: {formatCost(model.input_cost_per_mtok)}</span>
              <span>Out: {formatCost(model.output_cost_per_mtok)}</span>
            </div>
            <div className="model-id">{model.id}</div>
          </div>
        ))}

        {/* Custom model input for All tier */}
        {selectedTier === "All" && (
          <form className="model-custom-input" onSubmit={handleCustomModelSubmit}>
            <input
              type="text"
              placeholder="Enter model ID (e.g., provider/model-name)"
              value={customModelInput}
              onChange={(e) => setCustomModelInput(e.target.value)}
            />
            <button type="submit">Add</button>
          </form>
        )}
      </div>

      {/* Selected model summary */}
      {selectedModel && (
        <div className="model-picker-summary">
          <span>Selected:</span>
          <code>{selectedModel}</code>
        </div>
      )}
    </div>
  );
}

// Hook to manage model selection state
export function useModelPicker() {
  const [selectedModel, setSelectedModel] = useState("openai/gpt-4o");
  const [selectedTier, setSelectedTier] = useState<ModelTier>("Recommended");

  const handleTierChange = (tier: ModelTier) => {
    setSelectedTier(tier);
    // Auto-select first model in tier when switching
    const firstModel = MODELS[tier][0];
    if (firstModel) {
      setSelectedModel(firstModel.id);
    } else if (tier === "All") {
      setSelectedModel("");
    }
  };

  return {
    selectedModel,
    selectedTier,
    onModelSelect: setSelectedModel,
    onTierChange: handleTierChange,
  };
}
