import type { ReactNode } from 'react';

/**
 * 图标集：24×24 线性图标，统一 `currentColor` + 1.5 描边。
 *
 * **不引第三方图标库** —— 依赖白名单里只有 react / react-dom / @tauri-apps/api /
 * @tauri-apps/plugin-dialog，为了 13 个图标加一个库不划算；手写 SVG 还能精确控制
 * 描边粗细与圆角，保持和 ink/paper 视觉语言一致。
 */
export type IconName =
  | 'refresh'
  | 'folder'
  | 'folder-open'
  | 'folder-plus'
  | 'file-plus'
  | 'arrow-up'
  | 'arrow-left'
  | 'gear'
  | 'pencil'
  | 'trash'
  | 'copy'
  | 'external-link'
  | 'x-circle';

const PATHS: Record<IconName, ReactNode> = {
  // 刷新：断开的圆环 + 两个箭头
  refresh: (
    <>
      <polyline points="23 4 23 10 17 10" />
      <polyline points="1 20 1 14 7 14" />
      <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
    </>
  ),
  // 文件夹：选择目录
  folder: <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />,
  // 打开的文件夹：进入子目录
  'folder-open': (
    <path d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2" />
  ),
  // 文件夹 + 加号：新建文件夹
  'folder-plus': (
    <>
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      <path d="M12 11v6" />
      <path d="M9 14h6" />
    </>
  ),
  // 文件 + 加号：新建文件
  'file-plus': (
    <>
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <line x1="12" y1="18" x2="12" y2="12" />
      <line x1="9" y1="15" x2="15" y2="15" />
    </>
  ),
  // 向上箭头：返回上级目录
  'arrow-up': (
    <>
      <line x1="12" y1="20" x2="12" y2="5" />
      <polyline points="5 12 12 5 19 12" />
    </>
  ),
  // 向左箭头：从设置页返回上一个页面
  'arrow-left': (
    <>
      <line x1="19" y1="12" x2="5" y2="12" />
      <polyline points="12 19 5 12 12 5" />
    </>
  ),
  // 齿轮：打开设置
  gear: (
    <>
      <circle cx="12" cy="12" r="7" />
      <circle cx="12" cy="12" r="2.6" />
      <line x1="12" y1="2.5" x2="12" y2="5" />
      <line x1="12" y1="19" x2="12" y2="21.5" />
      <line x1="2.5" y1="12" x2="5" y2="12" />
      <line x1="19" y1="12" x2="21.5" y2="12" />
      <line x1="5.3" y1="5.3" x2="7.1" y2="7.1" />
      <line x1="16.9" y1="16.9" x2="18.7" y2="18.7" />
      <line x1="18.7" y1="5.3" x2="16.9" y2="7.1" />
      <line x1="7.1" y1="16.9" x2="5.3" y2="18.7" />
    </>
  ),
  // 铅笔：重命名
  pencil: <path d="M17 3a2.83 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" />,
  // 垃圾桶：删除
  trash: (
    <>
      <polyline points="3 6 5 6 21 6" />
      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
      <line x1="10" y1="11" x2="10" y2="17" />
      <line x1="14" y1="11" x2="14" y2="17" />
    </>
  ),
  // 两张叠放的纸：复制
  copy: (
    <>
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </>
  ),
  // 箭头出框：在 Finder 中显示
  'external-link': (
    <>
      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
      <polyline points="15 3 21 3 21 9" />
      <line x1="10" y1="14" x2="21" y2="3" />
    </>
  ),
  // 叉号圆圈：清空
  'x-circle': (
    <>
      <circle cx="12" cy="12" r="9" />
      <line x1="15" y1="9" x2="9" y2="15" />
      <line x1="9" y1="9" x2="15" y2="15" />
    </>
  ),
};

export interface IconProps {
  name: IconName;
  /** 像素尺寸，默认 15。 */
  size?: number;
  className?: string;
}

/**
 * 渲染一个图标。
 *
 * 装饰性（`aria-hidden`）—— 按钮本身通过 `aria-label` 提供无障碍名称，
 * 图标不重复朗读。
 */
export function Icon({ name, size = 15, className }: IconProps) {
  return (
    <svg
      className={className ? `icon ${className}` : 'icon'}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.7}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      {PATHS[name]}
    </svg>
  );
}
