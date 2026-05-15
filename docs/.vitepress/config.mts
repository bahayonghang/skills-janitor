import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'zh-CN',
  title: 'Skillscope',
  description: 'Skillscope CLI 与技能使用文档',
  lastUpdated: true,
  themeConfig: {
    nav: [
      { text: '首页', link: '/' },
      { text: 'CLI', link: '/cli' },
      { text: 'Skills', link: '/skills' }
    ],
    sidebar: [
      {
        text: '使用指南',
        items: [
          { text: '快速开始', link: '/' },
          { text: 'CLI 使用方法', link: '/cli' },
          { text: 'Skills 使用方法', link: '/skills' }
        ]
      }
    ],
    outline: {
      level: [2, 3],
      label: '本页目录'
    },
    docFooter: {
      prev: '上一页',
      next: '下一页'
    },
    search: {
      provider: 'local'
    }
  }
})
