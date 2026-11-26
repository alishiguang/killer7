import { useCallback, useEffect } from 'react';

import { Card, Checkbox, Icon, Kicker, Segmented } from '../components';
import { api } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { applyTheme } from '../lib/theme';
import { pushToast, setState, useAppStore } from '../lib/store';
import type { Accent, Settings as SettingsDto, ThemeMode } from '../lib/types';

export function Settings() {
  const { t } = useI18n();
  const [state] = useAppStore();
  const settings = state.settings;

  useEffect(() => {
    if (settings) {
      applyTheme(settings.themeMode, settings.accent);
    }
  }, [settings]);

  const patch = useCallback(async (delta: Partial<SettingsDto>) => {
    const next = await api.updateSettings(delta);
    applyTheme(next.themeMode, next.accent);
    setState({ settings: next });
  }, []);

  if (!settings) {
    return (
      <div className="page">
        <Kicker>{t('settings.title')}</Kicker>
        <div className="empty-state">{t('common.loading')}</div>
      </div>
    );
  }

  return (
    <div className="page">
      <div className="page__head">
        <div className="page__head-lead">
          <button
            type="button"
            className="native-button native-button--icon"
            title={t('common.back')}
            aria-label={t('common.back')}
            onClick={() => setState({ settingsOpen: false })}
          >
            <Icon name="arrow-left" />
          </button>
          <Kicker>{t('settings.title')}</Kicker>
        </div>
      </div>

      <div className="settings-grid">
        <Card title={t('settings.appearance')}>
          <div className="setting-row">
            <span className="setting-label">{t('settings.theme.mode')}</span>
            <Segmented<ThemeMode>
              value={settings.themeMode}
              onChange={(value) => void patch({ themeMode: value })}
              options={[
                { value: 'system', label: t('settings.theme.system') },
                { value: 'light', label: t('settings.theme.light') },
                { value: 'dark', label: t('settings.theme.dark') },
              ]}
            />
          </div>

          <div className="setting-row">
            <span className="setting-label">{t('settings.accent')}</span>
            <Segmented<Accent>
              value={settings.accent}
              onChange={(value) => void patch({ accent: value })}
              options={[
                { value: 'signal', label: t('settings.accent.signal') },
                { value: 'haze', label: t('settings.accent.haze') },
                { value: 'moss', label: t('settings.accent.moss') },
                { value: 'amber', label: t('settings.accent.amber') },
              ]}
            />
          </div>

          <div className="setting-row">
            <span className="setting-label">{t('settings.locale')}</span>
            <Segmented
              value={settings.locale}
              onChange={(value) => void patch({ locale: value })}
              options={[
                { value: 'zh-Hans', label: '简体中文' },
                { value: 'en', label: 'English' },
              ]}
            />
          </div>
        </Card>

        <Card title={t('settings.performance')}>
          <div className="setting-row">
            <span className="setting-label">{t('settings.refresh.interval')}</span>
            <Segmented
              value={String(settings.autoRefreshInterval)}
              onChange={(value) => void patch({ autoRefreshInterval: Number(value) })}
              options={[
                { value: '1', label: '1s' },
                { value: '5', label: '5s' },
                { value: '10', label: '10s' },
                { value: '30', label: '30s' },
              ]}
            />
          </div>

          <div className="setting-row">
            <span className="setting-label">{t('settings.processLimit')}</span>
            <Segmented
              value={String(settings.processListLimit)}
              onChange={(value) => void patch({ processListLimit: Number(value) })}
              options={[
                { value: '100', label: '100' },
                { value: '500', label: '500' },
                { value: '0', label: t('settings.noLimit') },
              ]}
            />
          </div>

          <div className="setting-row">
            <span className="setting-label">{t('settings.networkMonitor')}</span>
            <Checkbox
              checked={settings.enableNetworkMonitor}
              onChange={(checked) => void patch({ enableNetworkMonitor: checked })}
            />
          </div>
          <p className="setting-hint">{t('settings.performance.hint')}</p>
        </Card>

        <Card title={t('settings.startup')}>
          <div className="setting-row">
            <span className="setting-label">{t('settings.enableMenuBar')}</span>
            <Checkbox
              checked={settings.showMenuBar}
              onChange={(checked) => void patch({ showMenuBar: checked })}
            />
          </div>
          <p className="setting-hint">{t('settings.menuBar.hint')}</p>

          <div className="setting-row">
            <span className="setting-label">{t('settings.autoLaunch')}</span>
            <Checkbox
              checked={settings.launchAtLogin}
              onChange={async (checked) => {
                try {
                  await api.setAutoLaunch(checked);
                  await patch({ launchAtLogin: checked });
                } catch (error) {
                  pushToast('error', String(error));
                }
              }}
            />
          </div>
          <p className="setting-hint">{t('settings.autoLaunch.hint')}</p>

          <div className="setting-row">
            <span className="setting-label">{t('settings.globalShortcut')}</span>
            <Checkbox
              checked={settings.globalShortcutEnabled}
              onChange={(checked) => void patch({ globalShortcutEnabled: checked })}
            />
          </div>
          <p className="setting-hint">{t('settings.globalShortcut.hint')}</p>
        </Card>

        <Card title={t('settings.about')}>
          <dl className="about-meta">
            <dt>{t('settings.version')}</dt>
            <dd>1.2.0</dd>
            <dt>{t('settings.bundleId')}</dt>
            <dd>com.igigi.chuaker7</dd>
            <dt>{t('settings.copyright')}</dt>
            <dd>MIT</dd>
          </dl>
        </Card>
      </div>
    </div>
  );
}
