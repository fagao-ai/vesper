// 导入系统托盘
import { TrayIcon } from '@tauri-apps/api/tray';
// 获取当前窗口
import { getCurrentWindow } from '@tauri-apps/api/window';
// 托盘菜单
import { Menu } from '@tauri-apps/api/menu';
// 进程管理
import { exit } from '@tauri-apps/plugin-process';
// 图片和路径处理
import { Image } from '@tauri-apps/api/image';
import { resolveResource } from '@tauri-apps/api/path';

/**
 * 窗口置顶显示
 */
async function winShowFocus() {
    // 获取窗体实例
    const win = getCurrentWindow();
    // 检查窗口是否可见，如果不可见则显示出来
    if (!(await win.isVisible())) {
        await win.show();
    } else {
        // 检查是否处于最小化状态，如果处于最小化状态则解除最小化
        if (await win.isMinimized()) {
            await win.unminimize();
        }
        // 窗口置顶
        await win.setFocus();
    }
}

/**
 * 获取翻译文本
 */
function getTranslations() {
    const language = (typeof window !== 'undefined' ? localStorage.getItem('language') : 'zh') || 'zh';

    const translations = {
        zh: {
            show: '显示窗口',
            quit: '退出'
        },
        en: {
            show: 'Show Window',
            quit: 'Quit'
        }
    };

    return translations[language as 'zh' | 'en'] || translations.zh;
}

/**
 * 创建托盘菜单
 */
async function createMenu() {
    const t = getTranslations();

    return await Menu.new({
        // items 的显示顺序是倒过来的
        items: [
            {
                id: 'show',
                text: t.show,
                action: () => {
                    winShowFocus();
                }
            },
            {
                // 菜单 id
                id: 'quit',
                // 菜单文本
                text: t.quit,
                // 菜单项点击事件
                action: () => {
                    // 退出应用
                    exit(0);
                }
            }
        ]
    });
}

let trayInstance: Awaited<ReturnType<typeof TrayIcon.new>> | null = null;

/**
 * 创建系统托盘
 */
export async function createTray() {
    try {
        // 创建托盘菜单
        const menu = await createMenu();

        // 获取资源的绝对路径
        const resourcePath = await resolveResource('icons/32x32.png');

        // 创建 Image 对象
        const iconImage = await Image.fromPath(resourcePath);

        // 创建托盘图标配置
        const options = {
            // 托盘图标 - 使用 Image 对象
            icon: iconImage,
            // 托盘菜单
            menu: menu,
            // 托盘提示，悬浮在托盘图标上可以显示
            tooltip: 'Vesper SSH Tunnel Manager',
            // 是否在左键点击时显示托盘菜单
            menuOnLeftClick: false,
            // 托盘图标上事件的处理程序
            action: (event: any) => {
                // 左键点击事件
                if (event.type === 'Click' && event.button === "Left" && event.buttonState === 'Down') {
                    // 显示窗口
                    winShowFocus();
                }
            }
        };

        // 创建托盘
        trayInstance = await TrayIcon.new(options);
    } catch (error) {
        console.error('创建系统托盘失败:', error);
        throw error;
    }
}

/**
 * 更新托盘菜单语言
 */
export async function updateTrayLanguage() {
    if (!trayInstance) {
        return;
    }

    try {
        const menu = await createMenu();
        await trayInstance.setMenu(menu);
    } catch (error) {
        console.error('更新托盘菜单语言失败:', error);
    }
}
