import { useEffect, useRef } from 'react';

export interface SparklineProps {
  /** 采样序列。 */
  values: number[];
  /** 画布宽（CSS px）。 */
  width?: number;
  /** 画布高（CSS px）。 */
  height?: number;
  /** 线条颜色，缺省取当前强调色。 */
  color?: string;
  /** 上限。 */
  max?: number;
}

/** Canvas 手绘曲线，避免 SVG 逐点重排。 */
export function Sparkline({ values, width = 240, height = 48, color, max = 100 }: SparklineProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      return;
    }

    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.round(width * dpr);
    canvas.height = Math.round(height * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);

    if (values.length < 2) {
      return;
    }

    const stroke = color ?? getComputedStyle(canvas).getPropertyValue('--accent').trim() ?? '#F43D3F';
    const step = width / (values.length - 1);
    const y = (value: number) => height - (Math.min(value, max) / max) * height;

    ctx.beginPath();
    ctx.moveTo(0, y(values[0]));
    values.forEach((value, index) => {
      ctx.lineTo(index * step, y(value));
    });
    ctx.strokeStyle = stroke;
    ctx.lineWidth = 1.25;
    ctx.stroke();

    ctx.lineTo(width, height);
    ctx.lineTo(0, height);
    ctx.closePath();
    ctx.fillStyle = `${stroke}1f`;
    ctx.fill();
  }, [values, width, height, color, max]);

  return <canvas ref={canvasRef} className="sparkline" style={{ width, height }} />;
}
