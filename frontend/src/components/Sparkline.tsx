import { useEffect, useRef, useState } from 'react';

export interface SparklineProps {
  /** 采样序列。 */
  values: number[];
  /** 画布宽（CSS px）；缺省时自动跟随容器宽度。 */
  width?: number;
  /** 画布高（CSS px）；缺省时自动跟随容器高度。 */
  height?: number;
  /** 线条颜色，缺省取当前强调色。 */
  color?: string;
  /** 量程上限；缺省时按序列峰值自适应。 */
  max?: number;
}

/**
 * Canvas 手绘曲线，避免 SVG 逐点重排。
 *
 * **尺寸跟随容器**：卡片有内边距，写死像素尺寸会让画布溢出卡片边界
 * （内边距被吃掉之后还会压到描边上）。用 ResizeObserver 量容器内容宽高，
 * 容器变化时自动重绘。
 *
 * **量程自适应**：缺省按序列峰值缩放。固定 100% 量程的话，日常 10% 上下的
 * CPU 负载会被压成贴着底边的一条直线，看不出起伏 —— 这也是 macOS
 * 「活动监视器」CPU 负载图的画法。
 */
export function Sparkline({ values, width, height, color, max }: SparklineProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [measured, setMeasured] = useState({ width: 0, height: 0 });

  useEffect(() => {
    const canvas = canvasRef.current;
    const parent = canvas?.parentElement;
    if (!parent) {
      return;
    }
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) {
        return;
      }
      const nextWidth = width === undefined ? Math.floor(entry.contentRect.width) : 0;
      const nextHeight = height === undefined ? Math.floor(entry.contentRect.height) : 0;
      setMeasured((prev) =>
        prev.width === nextWidth && prev.height === nextHeight
          ? prev
          : { width: nextWidth, height: nextHeight },
      );
    });
    observer.observe(parent);
    return () => observer.disconnect();
  }, [width, height]);

  const drawWidth = width ?? measured.width;
  const drawHeight = height ?? (measured.height > 0 ? measured.height : 48);

  // 量程窗口：
  // - 显式传入 max 时用 [0, max]（百分比类指标）；
  // - 否则按序列自身的峰值自适应：以均值为中心，取一个不小于序列跨度的窗口，
  //   两端各留出半个跨度的呼吸空间。这样日常 10% 上下的 CPU 负载也能看出起伏，
  //   而恒定序列会是一条居中的水平线，而不是贴边的一块色块。
  const ceiling = max ?? 100;
  const floor = max ?? 0;
  let windowLow = floor;
  let windowHigh = ceiling;
  if (max === undefined && values.length > 0) {
    const lo = Math.min(...values);
    const hi = Math.max(...values);
    const center = (lo + hi) / 2;
    const half = Math.max(hi - lo, 1, Math.abs(center) * 0.02);
    windowLow = center - half;
    windowHigh = center + half;
  }
  const span = Math.max(windowHigh - windowLow, 0.0001);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || drawWidth <= 0 || drawHeight <= 0) {
      return;
    }
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      return;
    }

    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.round(drawWidth * dpr);
    canvas.height = Math.round(drawHeight * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, drawWidth, drawHeight);

    if (values.length < 2) {
      return;
    }

    const stroke = color ?? getComputedStyle(canvas).getPropertyValue('--accent').trim() ?? '#F43D3F';
    const step = drawWidth / (values.length - 1);
    const y = (value: number) =>
      drawHeight - ((Math.min(Math.max(value, windowLow), windowHigh) - windowLow) / span) * drawHeight;

    ctx.beginPath();
    ctx.moveTo(0, y(values[0]));
    values.forEach((value, index) => {
      ctx.lineTo(index * step, y(value));
    });
    ctx.strokeStyle = stroke;
    ctx.lineWidth = 1.25;
    ctx.stroke();

    ctx.lineTo(drawWidth, drawHeight);
    ctx.lineTo(0, drawHeight);
    ctx.closePath();
    ctx.fillStyle = `${stroke}1f`;
    ctx.fill();
  }, [values, drawWidth, drawHeight, color, windowLow, windowHigh, span]);

  return (
    <canvas
      ref={canvasRef}
      className="sparkline"
      style={{
        width: width === undefined ? '100%' : width,
        height: height === undefined ? '100%' : height,
      }}
    />
  );
}
