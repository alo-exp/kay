interface Props {
  totalCostUsd: number;
  totalTokensIn: number;
  totalTokensOut: number;
}

export function CostMeter({ totalCostUsd, totalTokensIn, totalTokensOut }: Props) {
  return (
    <div
      data-testid="cost-meter"
      style={{
        display: 'flex',
        gap: 16,
        color: 'var(--text-secondary)',
        fontSize: 12,
        fontFamily: 'var(--font-mono)',
      }}
    >
      <span title="Tokens in">{totalTokensIn.toLocaleString()}↑</span>
      <span title="Tokens out">{totalTokensOut.toLocaleString()}↓</span>
      <span title="USD cost" style={{ color: 'var(--text-primary)' }}>
        ${totalCostUsd.toFixed(4)}
      </span>
    </div>
  );
}
