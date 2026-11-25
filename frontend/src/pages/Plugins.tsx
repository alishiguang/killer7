import { useCallback, useEffect, useState } from 'react';

import { Card, Kicker } from '../components';
import { api } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { pushToast } from '../lib/store';
import type { PluginRow } from '../lib/types';

export function Plugins() {
  const { t } = useI18n();
  const [plugins, setPlugins] = useState<PluginRow[]>([]);
  const [dir, setDir] = useState('');

  useEffect(() => {
    void api.listPlugins().then(setPlugins);
    void api.pluginsDir().then(setDir);
  }, []);

  const reload = useCallback(async () => {
    const list = await api.reloadPlugins();
    setPlugins(list);
    pushToast('info', t('plugin.manifest'));
  }, [t]);

  return (
    <div className="page">
      <div className="page__head">
        <Kicker index="05">{t('nav.plugins')}</Kicker>
        <div className="page__tools">
          <button type="button" className="native-button" onClick={() => void reload()}>
            {t('process.refresh')}
          </button>
        </div>
      </div>

      <Card title={t('settings.plugin.dir')} className="page__card">
        <div className="plugin-dir">{dir || '—'}</div>

        {plugins.length === 0 ? (
          <div className="empty-state">{t('settings.plugin.empty')}</div>
        ) : (
          plugins.map((plugin) => (
            <div key={plugin.id} className="plugin-item">
              <div className="plugin-item__name">
                <span className="file-name__glyph" aria-hidden="true">
                  ◆
                </span>
                {plugin.name}
              </div>
              <dl className="plugin-item__meta">
                <dt>{t('plugin.version')}</dt>
                <dd>{plugin.version}</dd>
                <dt>{t('plugin.author')}</dt>
                <dd>{plugin.author ?? '—'}</dd>
                <dt>{t('plugin.permissions')}</dt>
                <dd>{plugin.permissions.length > 0 ? plugin.permissions.join('、') : '—'}</dd>
                <dt>{t('plugin.minVersion')}</dt>
                <dd>{plugin.minimumAppVersion ?? '—'}</dd>
              </dl>
            </div>
          ))
        )}
      </Card>
    </div>
  );
}
