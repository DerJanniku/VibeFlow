import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export async function checkForUpdates() {
  try {
    const update = await check();
    if (update) {
      console.log(`Update available: ${update.version} from ${update.date}`);
      
      const shouldUpdate = confirm(`A new version (${update.version}) is available. Would you like to update now?`);
      
      if (shouldUpdate) {
        console.log('Downloading update...');
        await update.downloadAndInstall();
        console.log('Update installed, relaunching...');
        await relaunch();
      }
    } else {
      console.log('No updates available.');
    }
  } catch (error) {
    console.error('Failed to check for updates:', error);
  }
}
