"use client"

interface RiskIndicatorProps {
  level: "low" | "medium" | "high"
  showLabel?: boolean
}

const config = {
  low: {
    dotClass: "bg-green-500",
    bgClass: "bg-green-500/15 text-green-400 border-green-500/30",
    label: "Low Risk",
  },
  medium: {
    dotClass: "bg-yellow-500",
    bgClass: "bg-yellow-500/15 text-yellow-400 border-yellow-500/30",
    label: "Medium",
  },
  high: {
    dotClass: "bg-red-500",
    bgClass: "bg-red-500/15 text-red-400 border-red-500/30",
    label: "High Risk",
  },
}

export default function RiskIndicator({ level, showLabel = true }: RiskIndicatorProps) {
  const { dotClass, bgClass, label } = config[level]

  if (!showLabel) {
    return (
      <span className={`inline-block h-2 w-2 rounded-full ${dotClass}`} aria-label={label} />
    )
  }

  return (
    <span className={`inline-flex items-center gap-1.5 px-2 py-0.5 text-xs font-medium rounded border ${bgClass}`}>
      <span className={`h-1.5 w-1.5 rounded-full ${dotClass}`} />
      {label}
    </span>
  )
}
