/** 日期格式化工具 */

/** 格式化为 YYYY-MM-DD HH:mm:ss */
export function formatDateTime(value?: string | null): string {
  if (!value) return '-';
  return value.replace('T', ' ').slice(0, 19);
}

/** 格式化为 YYYY-MM-DD */
export function formatDate(value?: string | null): string {
  if (!value) return '-';
  return value.slice(0, 10);
}

/** 相对时间描述 */
export function timeAgo(value?: string | null): string {
  if (!value) return '-';
  const t = new Date(value).getTime();
  if (Number.isNaN(t)) return value;
  const diff = Date.now() - t;
  const minutes = Math.floor(diff / 60000);
  if (minutes < 1) return '刚刚';
  if (minutes < 60) return `${minutes} 分钟前`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} 小时前`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days} 天前`;
  return formatDate(value);
}
