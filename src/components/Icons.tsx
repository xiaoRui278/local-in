interface IconProps {
  width?: number;
  height?: number;
  className?: string;
  color?: string;
}

export function SunIcon({ width = 18, height = 18, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <circle cx="12" cy="12" r="5" />
      <line x1="12" y1="1" x2="12" y2="3" />
      <line x1="12" y1="21" x2="12" y2="23" />
      <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
      <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
      <line x1="1" y1="12" x2="3" y2="12" />
      <line x1="21" y1="12" x2="23" y2="12" />
      <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
      <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
    </svg>
  );
}

export function MoonIcon({ width = 18, height = 18, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
    </svg>
  );
}

export function SettingsIcon({ width = 18, height = 18, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}

export function UsersIcon({ width = 18, height = 18, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
      <circle cx="9" cy="7" r="4" />
      <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
      <path d="M16 3.13a4 4 0 0 1 0 7.75" />
    </svg>
  );
}

export function BellIcon({ width = 16, height = 16, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" />
      <path d="M13.73 21a2 2 0 0 1-3.46 0" />
    </svg>
  );
}

export function SendIcon({ width = 18, height = 18, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <line x1="22" y1="2" x2="11" y2="13" />
      <polygon points="22 2 15 22 11 13 2 9 22 2" />
    </svg>
  );
}

export function PaperclipIcon({ width = 20, height = 20, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48" />
    </svg>
  );
}

export function CloseIcon({ width = 18, height = 18, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <circle cx="12" cy="12" r="10" />
      <line x1="15" y1="9" x2="9" y2="15" />
      <line x1="9" y1="9" x2="15" y2="15" />
    </svg>
  );
}

export function LogoutIcon({ width = 18, height = 18, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
      <polyline points="16 17 21 12 16 7" />
      <line x1="21" y1="12" x2="9" y2="12" />
    </svg>
  );
}

export function ChatIcon({ width = 14, height = 14, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
    </svg>
  );
}

export function ChatLargeIcon({ width = 64, height = 64, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="1" className={className} aria-hidden="true" opacity="0.2">
      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
    </svg>
  );
}

export function ChevronLeftIcon({ width = 16, height = 16, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <polyline points="15 18 9 12 15 6" />
    </svg>
  );
}

export function CopyIcon({ width = 14, height = 14, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </svg>
  );
}

export function FileIcon({ width = 24, height = 24, className, color = "currentColor" }: IconProps) {
  return (
    <svg width={width} height={height} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" className={className} aria-hidden="true">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <polyline points="14 2 14 8 20 8" />
    </svg>
  );
}
