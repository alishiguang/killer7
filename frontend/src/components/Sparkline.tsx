import { useEffect, useRef, useState } from 'react';

export interface SparklineProps {
  /** 采样序列。 */
  values: number[];
  /** 画布宽（CSS px）；缺省时自动跟随容器宽度。 */
  width?: number;
  /** 画布高（CSS px）。 */
  height?: number;
  /** 线条颜色，缺省取当前强调色。 */
  color?: string;
  /** 上限。 */
  max?: number;
}

/**
 * Canvas 手绘曲线，避免 SVG 逐点重排。
 *
 * 宽度默认**跟随容器**：卡片有内边距，写死像素宽度会让画布溢出卡片右边界
 * （内边距被吃掉之后还会压到描边上）。用 ResizeObserver 量容器内容宽度，
 * 容器变化时自动重绘。
 */
export function Sparkline({ values, width, height = 48, color, max = 100 }: SparklineProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [measured, setMeasured] = useState(0);

  useEffect(() => {
    if (width !== undefined) {
      return;
    }
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
      const next = Math.floor(entry.contentRect.width);
      if (next > 0) {
        setMeasured(next);
      }
    });
    observer.observe(parent);
    return () => observer.disconnect();
  }, [width]);

  const drawWidth = width ?? measured;

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || drawWidth <= 0) {
      return;
    }
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      return;
    }

    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.round(drawWidth * dpr);
    canvas.height = Math.round(height * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, drawWidth, height);

    if (values.length < 2) {
      return;
    }

    const stroke = color ?? getComputedStyle(canvas).getPropertyValue('--accent').trim() ?? '#F43D3F';
    const step = drawWidth / (values.length - 1);
    const y = (value: number) => height - (Math.min(value, max) / max) * height;

    ctx.beginPath();
    ctx.moveTo(0, y(values[0]));
    values.forEach((value, index) => {
      ctx.lineTo(index * step, y(value));
    });
    ctx.strokeStyle = stroke;
    ctx.lineWidth = 1.25;
    ctx.stroke();

    ctx.lineTo(drawWidth, height);
    ctx.lineTo(0, height);
    ctx.closePath();
    ctx.fillStyle = `${stroke}1f`;
    ctx.fill();
  }, [values, drawWidth, height, color, max]);

  return (
    <canvas
      ref={canvasRef}
      className="sparkline"
      style={{ width: width === undefined ? '100%' : width, height }}
    />
  );
}
