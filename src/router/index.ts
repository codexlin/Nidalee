import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { APP_ROUTES } from './appRoutes'
import { isOverlayWindow, overlayRoute } from '@/shared/utils/overlayWindow'

const productionRoutes: RouteRecordRaw[] = [
  {
    path: '/forbidden',
    name: 'forbidden',
    component: () => import('@/views/ForbiddenView.vue')
  },
  {
    path: APP_ROUTES.overview.path,
    name: APP_ROUTES.overview.name,
    component: () => import('../views/OverviewView.vue')
  },
  {
    path: APP_ROUTES.liveAnalysis.path,
    name: APP_ROUTES.liveAnalysis.name,
    component: () => import('../views/LiveAnalysisView.vue')
  },
  {
    path: APP_ROUTES.matchSearch.path,
    name: APP_ROUTES.matchSearch.name,
    component: () => import('../views/MatchSearchView.vue')
  },
  {
    path: APP_ROUTES.buildCenter.path,
    name: APP_ROUTES.buildCenter.name,
    component: () => import('../views/BuildCenterView.vue')
  },
  {
    path: APP_ROUTES.settings.path,
    name: APP_ROUTES.settings.name,
    component: () => import('../views/SettingsView.vue')
  },
  {
    path: '/augment-side-panel',
    name: 'augment-side-panel',
    component: () => import('../views/AugmentSidePanelView.vue'),
    meta: { shell: 'overlay' }
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

router.beforeEach((to) => {
  if (!isOverlayWindow()) return true
  const target = overlayRoute()
  if (to.path !== target) {
    return { path: target, replace: true }
  }
  return true
})

export default router
