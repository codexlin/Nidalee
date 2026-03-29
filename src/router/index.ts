import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/forbidden',
      name: 'Forbidden',
      component: () => import('@/views/ForbiddenView.vue')
    },
    {
      path: '/',
      name: 'dashboard',
      component: () => import('../views/DashboardView.vue')
    },
    {
      path: '/game-helper',
      name: 'game-helper',
      component: () => import('../views/GameHelperView.vue')
    },
    {
      path: '/match-analysis',
      name: 'match-analysis',
      component: () => import('../views/MatchAnalysisView.vue')
    },
    {
      path: '/match-search',
      name: 'match-search',
      component: () => import('../views/MatchSearchView.vue')
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/SettingsView.vue')
    },
    {
      path: '/opgg',
      name: 'opgg',
      component: () => import('../views/OpggView.vue')
    },
    {
      path: '/data-collection-test',
      name: 'data-collection-test',
      component: () => import('../views/DataCollectionTestView.vue')
    },
    {
      path: '/lobby-test',
      name: 'lobby-test',
      component: () => import('../features/lobby-test/LobbyTest.vue')
    }
  ]
})

export default router
