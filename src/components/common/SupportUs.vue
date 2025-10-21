<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger
} from '@/components/ui/dialog'
import { Heart, Home } from 'lucide-vue-next'

// 假设你的二维码图片放在这个路径
import wechatQr from '@/assets/wechat-qr.png' // 你已确认添加
import alipayQr from '@/assets/alipay-qr.png' // 你已确认添加

const GITHUB_SPONSOR_URL = 'https://github.com/sponsors/codexlin'
const PERSONAL_WEBSITE_URL = 'https://codexlin.com'
const AIFADIAN_URL = 'https://aifadian.net/codexlin'

function openLink(url: string) {
  window.open(url, '_blank')
}
</script>

<template>
  <div class="mt-8 p-6 border rounded-lg">
    <h3 class="text-lg font-medium flex items-center">
      <Heart class="w-5 h-5 mr-2 text-red-500" />
      支持我们
    </h3>
    <p class="text-sm text-muted-foreground mt-2">
      Nidalee 是一个免费的开源项目。如果它对你有帮助，可以考虑请开发者喝杯咖啡，这将是巨大的鼓励！
    </p>
    <div class="flex flex-wrap gap-4 mt-4">
      <Button @click="openLink(GITHUB_SPONSOR_URL)">
        <img
          src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/github.svg"
          class="w-4 h-4 mr-2"
          alt="GitHub"
        />
        GitHub Sponsors
      </Button>
      <Button @click="openLink(AIFADIAN_URL)" variant="secondary"> 爱发电 </Button>

      <Dialog>
        <DialogTrigger as-child>
          <Button variant="secondary">微信</Button>
        </DialogTrigger>
        <DialogContent class="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle>微信扫码支持</DialogTitle>
            <DialogDescription> 感谢您的支持！ </DialogDescription>
          </DialogHeader>
          <div class="flex justify-center p-4">
            <img :src="wechatQr" alt="WeChat QR Code" class="w-64 h-64" />
          </div>
        </DialogContent>
      </Dialog>

      <Dialog>
        <DialogTrigger as-child>
          <Button variant="secondary">支付宝</Button>
        </DialogTrigger>
        <DialogContent class="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle>支付宝扫码支持</DialogTitle>
            <DialogDescription> 感谢您的支持！ </DialogDescription>
          </DialogHeader>
          <div class="flex justify-center p-4">
            <img :src="alipayQr" alt="Alipay QR Code" class="w-64 h-64" />
          </div>
        </DialogContent>
      </Dialog>

      <Button @click="openLink(PERSONAL_WEBSITE_URL)" variant="outline">
        <Home class="w-4 h-4 mr-2" />
        开发者主页
      </Button>
    </div>
  </div>
</template>
