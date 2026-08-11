import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'

const productionRoutes: RouteRecordRaw[] = [
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
    redirect: { path: '/settings', query: { tab: 'game' } }
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
  }
]

const developmentRoutes: RouteRecordRaw[] = import.meta.env.DEV
  ? [
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
  : []

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    ...productionRoutes,
    ...developmentRoutes,
    {
      path: '/:pathMatch(.*)*',
      redirect: '/'
    }
  ]
})

export default router
