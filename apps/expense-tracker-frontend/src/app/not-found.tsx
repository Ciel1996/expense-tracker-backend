export const dynamic = 'force-static';

export default function NotFound() {
  return (
    <div className="min-h-screen flex flex-col items-center justify-center gap-2 p-6 text-center">
      <h1 className="text-2xl font-semibold">Page not found</h1>
      <p className="text-gray-600">The page you are looking for does not exist.</p>
      <a href="/" className="text-blue-600 hover:underline">Go back home</a>
    </div>
  );
}
