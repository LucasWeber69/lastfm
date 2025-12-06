import { useEffect } from 'react';
import { Outlet, useLocation } from 'react-router-dom';
import { Navbar } from '@/components/layout/Navbar';
import { BottomNav } from '@/components/layout/BottomNav';
import { useAuthStore } from '@/stores/authStore';

export function App() {
  const location = useLocation();
  const initialize = useAuthStore((state) => state.initialize);

  useEffect(() => {
    initialize();
  }, [initialize]);

  const isAuthPage = ['/login', '/register', '/'].includes(location.pathname);

  return (
    <div className="min-h-screen bg-background">
      {!isAuthPage && <Navbar />}
      <main className="min-h-screen">
        <Outlet />
      </main>
      {!isAuthPage && <BottomNav />}
    </div>
  );
}
