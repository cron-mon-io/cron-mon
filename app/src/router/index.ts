import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import MonitorsView from '@/views/MonitorsView.vue'
import MonitorView from '@/views/MonitorView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/docs',
      name: 'docs',
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/DocsView.vue')
    },
    {
      path: '/monitors',
      children: [
        {
          path: '',
          name: 'monitors',
          component: MonitorsView
        },
        {
          path: ':id',
          name: 'monitor',
          component: MonitorView
        }
      ]
    }
  ]
})

export default router
