import { useEffect, useState } from 'react';

import { Card, Kicker, Sparkline } from '../components';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { formatPercent, formatRate } from '../lib/format';
import type { SystemInfoDto } from '../lib/types';

const EMPTY: SystemInfoDto = {
  cpuUsage: 0,
  memoryUsage: 0,
  diskUsage: 0,
  networkUpload: 0,
  networkDownload: 0,
  cpuHistory: [],
  memoryHistory: [],
};

export function Monitor() {
  const { t } = useI18n();
  const [info, setInfo] = useState<SystemInfoDto>(EMPTY);

  useEffect(() => {
    let disposed = false;
    void api.systemSnapshot().then((snapshot) => {
      if (!disposed) {
        setInfo(snapshot);
      }
    });
    void onEvent<SystemInfoDto>(EVENT.systemInfo, (snapshot) => setInfo(snapshot));
    return () => {
      disposed = true;
    };
  }, []);

  return (
    <div className="page">
      <div className="page__head">
        <Kicker index="04">{t('monitor.title')}</Kicker>
      </div>

      <div className="metric-grid">
        <Metric label={t('monitor.cpu')} value={formatPercent(info.cpuUsage)} />
        <Metric label={t('monitor.memory')} value={formatPercent(info.memoryUsage)} />
        <Metric label={t('monitor.disk')} value={formatPercent(info.diskUsage)} />
        <Metric label={t('monitor.upload')} value={formatRate(info.networkUpload)} />
        <Metric label={t('monitor.download')} value={formatRate(info.networkDownload)} />
      </div>

      <div className="chart-grid">
        <Card title={t('monitor.cpu.history')}>
          <Sparkline values={info.cpuHistory} width={420} height={72} />
        </Card>
        <Card title={t('monitor.memory.history')}>
          <Sparkline values={info.memoryHistory} width={420} height={72} />
        </Card>
      </div>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="card metric">
      <div className="metric__label">{label}</div>
      <div className="metric__value">{value}</div>
    </div>
  );
}
