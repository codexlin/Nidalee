import { invoke } from '@tauri-apps/api/core'
import { toPng } from 'html-to-image'
import { toast } from 'vue-sonner'

const waitFrames = async (count = 2) => {
  for (let i = 0; i < count; i++) {
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()))
  }
}

const waitForImages = async (root: HTMLElement) => {
  const images = Array.from(root.querySelectorAll('img'))
  await Promise.all(
    images.map(
      (img) =>
        new Promise<void>((resolve) => {
          if (img.complete && img.naturalWidth > 0) {
            resolve()
            return
          }
          const done = () => resolve()
          img.addEventListener('load', done, { once: true })
          img.addEventListener('error', done, { once: true })
        })
    )
  )
}

const dataUrlToBase64 = (dataUrl: string) => {
  const idx = dataUrl.indexOf(',')
  return idx >= 0 ? dataUrl.slice(idx + 1) : dataUrl
}

export function useDashboardPosterExport() {
  const exporting = ref(false)

  const exportPoster = async (root: HTMLElement | null | undefined, fileStem: string) => {
    if (!root || exporting.value) return
    exporting.value = true
    try {
      await document.fonts.ready
      await waitForImages(root)
      await waitFrames(2)

      const bg = getComputedStyle(root).backgroundColor || getComputedStyle(document.body).backgroundColor
      const dataUrl = await toPng(root, {
        cacheBust: true,
        pixelRatio: 2,
        backgroundColor: bg && bg !== 'rgba(0, 0, 0, 0)' && bg !== 'transparent' ? bg : '#0a0a0a'
      })

      const pngBase64 = dataUrlToBase64(dataUrl)
      const defaultName = fileStem.endsWith('.png') ? fileStem : `${fileStem}.png`

      // 先复制剪贴板，再弹另存为（取消保存仍保留剪贴板）
      await invoke('copy_png_to_clipboard', { pngBase64 })
      const savedPath = await invoke<string | null>('save_png_file', {
        pngBase64,
        defaultName
      })

      if (savedPath) {
        toast.success('已保存并复制到剪贴板')
      } else {
        toast.success('已复制到剪贴板（未保存文件）')
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      toast.error(`海报导出失败：${message}`)
    } finally {
      exporting.value = false
    }
  }

  return { exporting, exportPoster }
}
