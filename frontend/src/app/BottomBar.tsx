import { useEffect, useState } from 'react';

import { Sparkline } from '../components';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import type { SystemInfoDto } from '../lib/types';

const EMPTY: SystemInfoDto = {
  cpuUsage: 0,
  memoryUsage: 0,
  diskUsage: 0,
  networkUpload: 0,
  networkDownload: 0,
  cpuHistory: [],
  memoryHistory: [],
  processCount: 0,
};

/**
 * 底部汇总栏：参考 macOS「活动监视器」窗口底部的三栏面板。
 *
 * 左栏是系统级百分比，中栏是 CPU 负载曲线，右栏是计数与网络速率。
 * 数据全部来自后端已经在推的 `system-info` 事件 —— 底栏自己订阅一份，
 * 不用让页面把自己的状态往外传。
 */
export function BottomBar() {
  const { t } = useI18n();
  const [info, setInfo] = useState<SystemInfoDto>(EMPTY);

  useEffect(() => {
    let disposed = false;

    void api.systemSnapshot().then((snapshot) => {
      if (!disposed) {
        setInfo(snapshot);
      }
    });

    void onEvent<SystemInfoDto>(EVENT.systemInfo, (next) => {
      if (!disposed) {
        setInfo(next);
      }
    });

    return () => {
      disposed = true;
    };
  }, []);

  return (
    <div className="bottombar">
      <div className="bottombar__panel">
        <div className="bottombar__zone">
          <div className="bottombar__row">
            <span className="bottombar__label">{t('bottom.cpu')}</span>
            <span className="bottombar__value bottombar__value--accent">
              {info.cpuUsage.toFixed(1)}%
            </span>
          </div>
          <div className="bottombar__row">
            <span className="bottombar__label">{t('bottom.memory')}</span>
            <span className="bottombar__value">{info.memoryUsage.toFixed(1)}%</span>
          </div>
          <div className="bottombar__row">
            <span className="bottombar__label">{t('bottom.disk')}</span>
            <span className="bottombar__value">{info.diskUsage.toFixed(1)}%</span>
          </div>
        </div>

        <div className="bottombar__zone bottombar__zone--chart">
          <div className="bottombar__row">
            <span className="bottombar__label">{t('bottom.cpuLoad')}</span>
          </div>
          <div className="bottombar__chart">
            <Sparkline values={info.cpuHistory} height={28} />
          </div>
        </div>

        <div className="bottombar__zone">
          <div className="bottombar__row">
            <span className="bottombar__label">{t('bottom.processes')}</span>
            <span className="bottombar__value">{info.processCount}</span>
          </div>
          <div className="bottombar__row">
            <span className="bottombar__label">{t('bottom.network')}</span>
            <span className="bottombar__value">
              ↑ {info.networkUpload.toFixed(1)} · ↓ {info.networkDownload.toFixed(1)}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
